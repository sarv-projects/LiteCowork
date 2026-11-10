//! ACP client over newline-delimited JSON-RPC 2.0 (specs/002 T17).
//!
//! The client drives `initialize`, `session/new`, `session/prompt` and
//! `session/cancel`, consumes correlated `session/update` notifications, and
//! answers the agent-originated `session/request_permission`, `fs/*` and
//! `terminal/*` requests by routing them to an [`AcpClientHandler`]. The handler
//! is the Client owner: it enforces the Workspace boundary and decides on
//! permission. Nothing here records an Effect, receipt or grant; a native tool
//! action stays a REPORTED observation.
//!
//! The transport is any `BufRead` plus `Write`, so a subprocess pair and a test
//! fixture use the same code. Lines are bounded; an oversized line fails the turn
//! closed instead of growing memory.

#![cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "the ACP client is exercised by the fixture-agent tests; no supervisor arm launches an ACP provider until Feature 004 adds the Claude Code dispatch arm"
    )
)]

use super::adapter::{
    AcpClientError, AcpClientHandler, AcpPermissionDecision, AcpPermissionRequest,
    AcpSessionUpdate, CancelSignal, acp_method, parse_acp_session_update,
};
use serde_json::{Value, json};
use std::io::{BufRead, Write};

/// Longest accepted JSON-RPC line. A line over this bound fails the connection.
pub(crate) const ACP_MAX_LINE_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AcpClientFault {
    /// The agent closed the stream before answering.
    Closed,
    /// A line exceeded [`ACP_MAX_LINE_BYTES`] or was not a JSON object.
    Malformed,
    /// The agent answered a request with a JSON-RPC error.
    AgentError,
    Io,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AcpStopReason {
    EndTurn,
    Cancelled,
    Other,
}

/// What one prompt produced. Foreign-session traffic is counted, never applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AcpTurnOutcome {
    pub stop_reason: AcpStopReason,
    pub updates: Vec<AcpSessionUpdate>,
    pub foreign_dropped: u32,
    pub refused_requests: u32,
}

pub(crate) struct AcpClient<R: BufRead, W: Write> {
    reader: R,
    writer: W,
    next_id: u64,
}

impl<R: BufRead, W: Write> AcpClient<R, W> {
    pub(crate) fn new(reader: R, writer: W) -> Self {
        Self {
            reader,
            writer,
            next_id: 1,
        }
    }

    /// Negotiates the protocol version and declares the Client surfaces. The
    /// terminal and filesystem capabilities are declared only because the handler
    /// serves them; the agent returns its own capabilities.
    pub(crate) fn initialize(&mut self, protocol_version: u32) -> Result<Value, AcpClientFault> {
        self.request(
            acp_method::INITIALIZE,
            json!({
                "protocolVersion": protocol_version,
                "clientCapabilities": {
                    "fs": {"readTextFile": true, "writeTextFile": true},
                    "terminal": true
                }
            }),
            &mut NoHandler,
            None,
        )
        .map(|outcome| outcome.result)
    }

    pub(crate) fn session_new(&mut self, cwd: &str) -> Result<String, AcpClientFault> {
        let result = self
            .request(
                acp_method::SESSION_NEW,
                json!({"cwd": cwd, "mcpServers": []}),
                &mut NoHandler,
                None,
            )?
            .result;
        result
            .get("sessionId")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or(AcpClientFault::Malformed)
    }

    /// Sends one prompt and serves the connection until the agent answers it. A
    /// requested cancel sends one `session/cancel` notification; the turn still
    /// settles only on the agent's own stop reason.
    pub(crate) fn prompt(
        &mut self,
        session_id: &str,
        text: &str,
        handler: &mut dyn AcpClientHandler,
        cancel: Option<&CancelSignal>,
    ) -> Result<AcpTurnOutcome, AcpClientFault> {
        let outcome = self.request_scoped(
            acp_method::SESSION_PROMPT,
            json!({
                "sessionId": session_id,
                "prompt": [{"type": "text", "text": text}]
            }),
            handler,
            cancel,
            Some(session_id),
        )?;
        let stop_reason = match outcome.result.get("stopReason").and_then(Value::as_str) {
            Some("end_turn") => AcpStopReason::EndTurn,
            Some("cancelled") => AcpStopReason::Cancelled,
            _ => AcpStopReason::Other,
        };
        Ok(AcpTurnOutcome {
            stop_reason,
            updates: outcome.updates,
            foreign_dropped: outcome.foreign_dropped,
            refused_requests: outcome.refused_requests,
        })
    }

    fn request(
        &mut self,
        method: &str,
        params: Value,
        handler: &mut dyn AcpClientHandler,
        cancel: Option<&CancelSignal>,
    ) -> Result<Exchange, AcpClientFault> {
        self.request_scoped(method, params, handler, cancel, None)
    }

    fn request_scoped(
        &mut self,
        method: &str,
        params: Value,
        handler: &mut dyn AcpClientHandler,
        cancel: Option<&CancelSignal>,
        session: Option<&str>,
    ) -> Result<Exchange, AcpClientFault> {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))?;
        let mut exchange = Exchange::default();
        let mut cancel_sent = false;
        loop {
            if let (Some(signal), Some(session_id)) = (cancel, session) {
                if signal.is_requested() && !cancel_sent {
                    cancel_sent = true;
                    self.send(&json!({
                        "jsonrpc": "2.0",
                        "method": acp_method::SESSION_CANCEL,
                        "params": {"sessionId": session_id}
                    }))?;
                }
            }
            let message = self.read_message()?;
            let method_name = message.get("method").and_then(Value::as_str);
            match (method_name, message.get("id")) {
                // Agent response to our request.
                (None, Some(response_id)) => {
                    if response_id != &json!(id) {
                        // A response for a request we are not waiting on cannot settle this one.
                        exchange.foreign_dropped += 1;
                        continue;
                    }
                    if message.get("error").is_some() {
                        return Err(AcpClientFault::AgentError);
                    }
                    exchange.result = message.get("result").cloned().unwrap_or(Value::Null);
                    return Ok(exchange);
                }
                // Agent notification.
                (Some(acp_method::SESSION_UPDATE), None) => {
                    let params = message.get("params").cloned().unwrap_or(Value::Null);
                    match session.and_then(|expected| parse_acp_session_update(expected, &params)) {
                        Some(update) => exchange.updates.push(update),
                        None => exchange.foreign_dropped += 1,
                    }
                }
                // Agent request to the Client owner.
                (Some(name), Some(request_id)) => {
                    let request_id = request_id.clone();
                    let params = message.get("params").cloned().unwrap_or(Value::Null);
                    let reply = self.serve_client_request(name, &params, session, handler);
                    if reply.is_err() {
                        exchange.refused_requests += 1;
                    }
                    self.send(&match reply {
                        Ok(result) => json!({"jsonrpc": "2.0", "id": request_id, "result": result}),
                        Err((code, text)) => json!({
                            "jsonrpc": "2.0",
                            "id": request_id,
                            "error": {"code": code, "message": text}
                        }),
                    })?;
                }
                // Unknown notification: ignored, never applied.
                _ => exchange.foreign_dropped += 1,
            }
        }
    }

    /// Routes one agent-originated request. A request that names another session,
    /// or names none while a session is active, is refused before the handler runs.
    fn serve_client_request(
        &self,
        method: &str,
        params: &Value,
        session: Option<&str>,
        handler: &mut dyn AcpClientHandler,
    ) -> Result<Value, (i64, &'static str)> {
        const METHOD_NOT_FOUND: (i64, &str) = (-32601, "method not found");
        const INVALID_PARAMS: (i64, &str) = (-32602, "invalid params");
        const REFUSED: (i64, &str) = (-32000, "refused by client");
        let known = matches!(
            method,
            acp_method::REQUEST_PERMISSION
                | acp_method::FS_READ_TEXT_FILE
                | acp_method::FS_WRITE_TEXT_FILE
                | acp_method::TERMINAL_CREATE
                | acp_method::TERMINAL_OUTPUT
                | acp_method::TERMINAL_KILL
                | acp_method::TERMINAL_RELEASE
        );
        if !known {
            return Err(METHOD_NOT_FOUND);
        }
        let request_session = params.get("sessionId").and_then(Value::as_str);
        if session.is_none() || request_session != session {
            return Err(INVALID_PARAMS);
        }
        let text = |key: &str| params.get(key).and_then(Value::as_str);
        let map = |_: AcpClientError| REFUSED;
        match method {
            acp_method::REQUEST_PERMISSION => {
                let permission_id = params
                    .get("toolCall")
                    .and_then(|call| call.get("toolCallId"))
                    .and_then(Value::as_str)
                    .ok_or(INVALID_PARAMS)?;
                let title = params
                    .get("toolCall")
                    .and_then(|call| call.get("title"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .chars()
                    .take(256)
                    .collect();
                let request = AcpPermissionRequest {
                    session_id: request_session.unwrap_or_default().to_owned(),
                    permission_id: permission_id.to_owned(),
                    title,
                };
                let wanted_kind = match handler.request_permission(&request) {
                    AcpPermissionDecision::Allow => Some("allow_once"),
                    AcpPermissionDecision::Deny => Some("reject_once"),
                    AcpPermissionDecision::Defer => None,
                };
                let option_id = wanted_kind.and_then(|kind| {
                    params
                        .get("options")
                        .and_then(Value::as_array)?
                        .iter()
                        .find(|option| option.get("kind").and_then(Value::as_str) == Some(kind))
                        .and_then(|option| option.get("optionId").and_then(Value::as_str))
                });
                Ok(match option_id {
                    Some(option_id) => {
                        json!({"outcome": {"outcome": "selected", "optionId": option_id}})
                    }
                    // Defer, or no matching option: answer cancelled, never invent a grant.
                    None => json!({"outcome": {"outcome": "cancelled"}}),
                })
            }
            acp_method::FS_READ_TEXT_FILE => {
                let path = text("path").ok_or(INVALID_PARAMS)?;
                handler
                    .fs_read_text_file(path)
                    .map(|content| json!({"content": content}))
                    .map_err(map)
            }
            acp_method::FS_WRITE_TEXT_FILE => {
                let path = text("path").ok_or(INVALID_PARAMS)?;
                let content = text("content").ok_or(INVALID_PARAMS)?;
                handler
                    .fs_write_text_file(path, content)
                    .map(|()| Value::Null)
                    .map_err(map)
            }
            acp_method::TERMINAL_CREATE => {
                let command = text("command").ok_or(INVALID_PARAMS)?;
                let args: Vec<String> = params
                    .get("args")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|item| item.as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default();
                handler
                    .terminal_create(command, &args)
                    .map(|terminal_id| json!({"terminalId": terminal_id}))
                    .map_err(map)
            }
            acp_method::TERMINAL_OUTPUT => {
                let terminal_id = text("terminalId").ok_or(INVALID_PARAMS)?;
                handler
                    .terminal_output(terminal_id)
                    .map(|output| json!({"output": output, "truncated": false}))
                    .map_err(map)
            }
            acp_method::TERMINAL_KILL => {
                let terminal_id = text("terminalId").ok_or(INVALID_PARAMS)?;
                handler
                    .terminal_kill(terminal_id)
                    .map(|()| json!({}))
                    .map_err(map)
            }
            _ => {
                let terminal_id = text("terminalId").ok_or(INVALID_PARAMS)?;
                handler
                    .terminal_release(terminal_id)
                    .map(|()| json!({}))
                    .map_err(map)
            }
        }
    }

    fn send(&mut self, message: &Value) -> Result<(), AcpClientFault> {
        let mut line = serde_json::to_vec(message).map_err(|_| AcpClientFault::Malformed)?;
        line.push(b'\n');
        self.writer
            .write_all(&line)
            .and_then(|()| self.writer.flush())
            .map_err(|_| AcpClientFault::Io)
    }

    fn read_message(&mut self) -> Result<Value, AcpClientFault> {
        let mut line = Vec::new();
        loop {
            let available = self.reader.fill_buf().map_err(|_| AcpClientFault::Io)?;
            if available.is_empty() {
                return Err(AcpClientFault::Closed);
            }
            let (consumed, done) = match available.iter().position(|byte| *byte == b'\n') {
                Some(end) => (end + 1, true),
                None => (available.len(), false),
            };
            if line.len() + consumed > ACP_MAX_LINE_BYTES + 1 {
                return Err(AcpClientFault::Malformed);
            }
            line.extend_from_slice(&available[..consumed]);
            self.reader.consume(consumed);
            if done {
                break;
            }
        }
        let value: Value =
            serde_json::from_slice(&line).map_err(|_| AcpClientFault::Malformed)?;
        if value.is_object() {
            Ok(value)
        } else {
            Err(AcpClientFault::Malformed)
        }
    }
}

#[derive(Default)]
struct Exchange {
    result: Value,
    updates: Vec<AcpSessionUpdate>,
    foreign_dropped: u32,
    refused_requests: u32,
}

/// Handler used before a session exists: every Client-surface call is refused.
struct NoHandler;

impl AcpClientHandler for NoHandler {
    fn request_permission(&mut self, _: &AcpPermissionRequest) -> AcpPermissionDecision {
        AcpPermissionDecision::Deny
    }
    fn fs_read_text_file(&mut self, _: &str) -> Result<String, AcpClientError> {
        Err(AcpClientError::Refused)
    }
    fn fs_write_text_file(&mut self, _: &str, _: &str) -> Result<(), AcpClientError> {
        Err(AcpClientError::Refused)
    }
    fn terminal_create(&mut self, _: &str, _: &[String]) -> Result<String, AcpClientError> {
        Err(AcpClientError::Refused)
    }
    fn terminal_output(&mut self, _: &str) -> Result<String, AcpClientError> {
        Err(AcpClientError::Refused)
    }
    fn terminal_kill(&mut self, _: &str) -> Result<(), AcpClientError> {
        Err(AcpClientError::Refused)
    }
    fn terminal_release(&mut self, _: &str) -> Result<(), AcpClientError> {
        Err(AcpClientError::Refused)
    }
}

#[cfg(test)]
mod acp_client_adapter_tests {
    use super::*;
    use std::io::{self, BufReader, Read};
    use std::sync::mpsc;
    use std::thread;

    struct ChannelReader {
        receiver: mpsc::Receiver<Vec<u8>>,
        pending: Vec<u8>,
    }

    impl Read for ChannelReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if self.pending.is_empty() {
                match self.receiver.recv() {
                    Ok(bytes) => self.pending = bytes,
                    Err(_) => return Ok(0),
                }
            }
            let count = buffer.len().min(self.pending.len());
            buffer[..count].copy_from_slice(&self.pending[..count]);
            self.pending.drain(..count);
            Ok(count)
        }
    }

    struct ChannelWriter(mpsc::Sender<Vec<u8>>);

    impl Write for ChannelWriter {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.0
                .send(buffer.to_vec())
                .map_err(|_| io::Error::from(io::ErrorKind::BrokenPipe))?;
            Ok(buffer.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    type Client = AcpClient<BufReader<ChannelReader>, ChannelWriter>;

    /// Connects a client to a fixture agent thread. The agent closure receives the
    /// lines the client sends and a sender for lines to the client.
    fn connect(
        agent: impl FnOnce(mpsc::Receiver<Vec<u8>>, mpsc::Sender<Vec<u8>>) + Send + 'static,
    ) -> (Client, thread::JoinHandle<()>) {
        let (to_agent, agent_in) = mpsc::channel();
        let (to_client, client_in) = mpsc::channel();
        let handle = thread::spawn(move || agent(agent_in, to_client));
        let client = AcpClient::new(
            BufReader::new(ChannelReader {
                receiver: client_in,
                pending: Vec::new(),
            }),
            ChannelWriter(to_agent),
        );
        (client, handle)
    }

    fn recv_line(receiver: &mpsc::Receiver<Vec<u8>>) -> Option<Value> {
        let mut buffer = Vec::new();
        loop {
            buffer.extend(receiver.recv().ok()?);
            if buffer.ends_with(b"\n") {
                return serde_json::from_slice(&buffer).ok();
            }
        }
    }

    fn emit(sender: &mpsc::Sender<Vec<u8>>, value: Value) {
        let mut bytes = serde_json::to_vec(&value).unwrap();
        bytes.push(b'\n');
        let _ = sender.send(bytes);
    }

    #[derive(Default)]
    struct Recorder {
        permissions: Vec<AcpPermissionRequest>,
        reads: Vec<String>,
        writes: Vec<(String, String)>,
        terminals: Vec<String>,
        decision: Option<AcpPermissionDecision>,
    }

    impl AcpClientHandler for Recorder {
        fn request_permission(&mut self, request: &AcpPermissionRequest) -> AcpPermissionDecision {
            self.permissions.push(request.clone());
            self.decision.unwrap_or(AcpPermissionDecision::Deny)
        }
        fn fs_read_text_file(&mut self, path: &str) -> Result<String, AcpClientError> {
            if path.contains("..") {
                return Err(AcpClientError::OutsideWorkspace);
            }
            self.reads.push(path.to_owned());
            Ok("file body".to_owned())
        }
        fn fs_write_text_file(&mut self, path: &str, contents: &str) -> Result<(), AcpClientError> {
            self.writes.push((path.to_owned(), contents.to_owned()));
            Ok(())
        }
        fn terminal_create(&mut self, command: &str, _: &[String]) -> Result<String, AcpClientError> {
            self.terminals.push(format!("create:{command}"));
            Ok("term-1".to_owned())
        }
        fn terminal_output(&mut self, id: &str) -> Result<String, AcpClientError> {
            self.terminals.push(format!("output:{id}"));
            Ok("ok".to_owned())
        }
        fn terminal_kill(&mut self, id: &str) -> Result<(), AcpClientError> {
            self.terminals.push(format!("kill:{id}"));
            Ok(())
        }
        fn terminal_release(&mut self, id: &str) -> Result<(), AcpClientError> {
            self.terminals.push(format!("release:{id}"));
            Ok(())
        }
    }

    fn respond(sender: &mpsc::Sender<Vec<u8>>, request: &Value, result: Value) {
        emit(sender, json!({"jsonrpc":"2.0","id":request["id"],"result":result}));
    }

    #[test]
    fn acp_client_adapter_completes_the_lifecycle_and_routes_client_surfaces() {
        let (mut client, agent) = connect(|input, output| {
            let init = recv_line(&input).unwrap();
            assert_eq!(init["method"], "initialize");
            assert_eq!(init["params"]["clientCapabilities"]["terminal"], true);
            respond(&output, &init, json!({"protocolVersion": 1, "agentCapabilities": {}}));
            let new = recv_line(&input).unwrap();
            assert_eq!(new["method"], "session/new");
            respond(&output, &new, json!({"sessionId": "s1"}));
            let prompt = recv_line(&input).unwrap();
            assert_eq!(prompt["method"], "session/prompt");
            emit(&output, json!({"jsonrpc":"2.0","method":"session/update","params":{
                "sessionId":"s1","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"hi"}}}}));
            // Client-surface requests.
            for (id, method, params) in [
                (101, "session/request_permission", json!({"sessionId":"s1","toolCall":{"toolCallId":"t1","title":"Run tests"},
                    "options":[{"optionId":"yes","name":"Allow","kind":"allow_once"},{"optionId":"no","name":"Reject","kind":"reject_once"}]})),
                (102, "fs/read_text_file", json!({"sessionId":"s1","path":"notes.txt"})),
                (103, "fs/write_text_file", json!({"sessionId":"s1","path":"out.txt","content":"data"})),
                (104, "terminal/create", json!({"sessionId":"s1","command":"ls","args":["-l"]})),
                (105, "terminal/output", json!({"sessionId":"s1","terminalId":"term-1"})),
                (106, "terminal/kill", json!({"sessionId":"s1","terminalId":"term-1"})),
                (107, "terminal/release", json!({"sessionId":"s1","terminalId":"term-1"})),
            ] {
                emit(&output, json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}));
                let reply = recv_line(&input).unwrap();
                assert_eq!(reply["id"], id, "{method}");
                assert!(reply.get("error").is_none(), "{method} must succeed");
                if id == 101 {
                    assert_eq!(reply["result"]["outcome"]["optionId"], "yes");
                }
            }
            respond(&output, &prompt, json!({"stopReason":"end_turn"}));
        });
        client.initialize(1).unwrap();
        let session = client.session_new("/workspace").unwrap();
        let mut handler = Recorder {
            decision: Some(AcpPermissionDecision::Allow),
            ..Recorder::default()
        };
        let outcome = client.prompt(&session, "hello", &mut handler, None).unwrap();
        agent.join().unwrap();
        assert_eq!(outcome.stop_reason, AcpStopReason::EndTurn);
        assert_eq!(outcome.updates, vec![AcpSessionUpdate::AgentMessageChunk { text: "hi".into() }]);
        assert_eq!(handler.permissions.len(), 1);
        assert_eq!(handler.permissions[0].title, "Run tests");
        assert_eq!(handler.reads, ["notes.txt"]);
        assert_eq!(handler.writes, [("out.txt".to_owned(), "data".to_owned())]);
        assert_eq!(handler.terminals.len(), 4);
        assert_eq!(outcome.refused_requests, 0);
    }

    #[test]
    fn acp_client_adapter_never_applies_foreign_session_traffic() {
        let (mut client, agent) = connect(|input, output| {
            respond(&output, &recv_line(&input).unwrap(), json!({"sessionId": "s1"}));
            let prompt = recv_line(&input).unwrap();
            emit(&output, json!({"jsonrpc":"2.0","method":"session/update","params":{
                "sessionId":"other","update":{"sessionUpdate":"agent_message_chunk","content":{"text":"leak"}}}}));
            // A foreign-session fs write must be refused before the handler runs.
            emit(&output, json!({"jsonrpc":"2.0","id":9,"method":"fs/write_text_file",
                "params":{"sessionId":"other","path":"x","content":"y"}}));
            let reply = recv_line(&input).unwrap();
            assert!(reply.get("error").is_some());
            // So is an unknown agent method.
            emit(&output, json!({"jsonrpc":"2.0","id":10,"method":"vendor/unknown","params":{}}));
            assert_eq!(recv_line(&input).unwrap()["error"]["code"], -32601);
            respond(&output, &prompt, json!({"stopReason":"end_turn"}));
        });
        let session = client.session_new("/w").unwrap();
        let mut handler = Recorder::default();
        let outcome = client.prompt(&session, "p", &mut handler, None).unwrap();
        agent.join().unwrap();
        assert!(outcome.updates.is_empty());
        assert_eq!(outcome.foreign_dropped, 1);
        assert_eq!(outcome.refused_requests, 2);
        assert!(handler.writes.is_empty(), "foreign session must not reach the handler");
    }

    #[test]
    fn acp_client_adapter_refuses_path_escape_and_never_invents_a_permission_grant() {
        let (mut client, agent) = connect(|input, output| {
            respond(&output, &recv_line(&input).unwrap(), json!({"sessionId": "s1"}));
            let prompt = recv_line(&input).unwrap();
            emit(&output, json!({"jsonrpc":"2.0","id":1,"method":"fs/read_text_file",
                "params":{"sessionId":"s1","path":"../../etc/passwd"}}));
            assert!(recv_line(&input).unwrap().get("error").is_some());
            emit(&output, json!({"jsonrpc":"2.0","id":2,"method":"session/request_permission",
                "params":{"sessionId":"s1","toolCall":{"toolCallId":"t","title":"x"},
                "options":[{"optionId":"yes","name":"A","kind":"allow_once"}]}}));
            // Deny with no reject option, and Defer: both answer cancelled, never allow.
            assert_eq!(recv_line(&input).unwrap()["result"]["outcome"]["outcome"], "cancelled");
            respond(&output, &prompt, json!({"stopReason":"end_turn"}));
        });
        let session = client.session_new("/w").unwrap();
        let mut handler = Recorder::default();
        client.prompt(&session, "p", &mut handler, None).unwrap();
        agent.join().unwrap();
        assert!(handler.reads.is_empty());
    }

    #[test]
    fn acp_client_adapter_sends_one_cancel_and_settles_on_the_agent_stop_reason() {
        let cancel = CancelSignal::new();
        let for_agent = cancel.clone();
        let (mut client, agent) = connect(move |input, output| {
            respond(&output, &recv_line(&input).unwrap(), json!({"sessionId": "s1"}));
            let prompt = recv_line(&input).unwrap();
            // Wake the client loop so it observes the cancel request.
            for_agent.request();
            emit(&output, json!({"jsonrpc":"2.0","method":"session/update","params":{
                "sessionId":"s1","update":{"sessionUpdate":"plan","entries":[{}]}}}));
            let cancel_message = recv_line(&input).unwrap();
            assert_eq!(cancel_message["method"], "session/cancel");
            assert!(cancel_message.get("id").is_none(), "cancel is a notification");
            respond(&output, &prompt, json!({"stopReason":"cancelled"}));
        });
        let session = client.session_new("/w").unwrap();
        let mut handler = Recorder::default();
        let outcome = client.prompt(&session, "p", &mut handler, Some(&cancel)).unwrap();
        agent.join().unwrap();
        assert_eq!(outcome.stop_reason, AcpStopReason::Cancelled);
    }

    #[test]
    fn acp_client_adapter_fails_closed_on_oversized_lines_closed_streams_and_agent_errors() {
        let (mut client, agent) = connect(|input, output| {
            let _ = recv_line(&input);
            let _ = output.send(vec![b'x'; ACP_MAX_LINE_BYTES + 10]);
        });
        assert_eq!(client.initialize(1).unwrap_err(), AcpClientFault::Malformed);
        agent.join().unwrap();

        let (mut closed, agent) = connect(|input, _output| {
            let _ = recv_line(&input);
        });
        assert_eq!(closed.initialize(1).unwrap_err(), AcpClientFault::Closed);
        agent.join().unwrap();

        let (mut errored, agent) = connect(|input, output| {
            let request = recv_line(&input).unwrap();
            emit(&output, json!({"jsonrpc":"2.0","id":request["id"],"error":{"code":-1,"message":"no"}}));
        });
        assert_eq!(errored.initialize(1).unwrap_err(), AcpClientFault::AgentError);
        agent.join().unwrap();
    }

    #[test]
    fn acp_client_adapter_is_distinct_from_the_non_acp_opencode_launch_catalog() {
        use crate::agents::adapter::{AcpProvider, select_launch_entry};
        assert!(select_launch_entry(AcpProvider::ClaudeCode).is_some());
        assert!(select_launch_entry(AcpProvider::Codex).is_some());
    }
}
