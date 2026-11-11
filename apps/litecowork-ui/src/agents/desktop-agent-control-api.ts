import { invoke } from "@tauri-apps/api/core";
import {
  parseAgentControlDescriptor,
  parseAgentAuthChallenge,
  parseNativeConfigOpenReceipt,
  type AgentControlActionsApi,
  type AgentControlDescriptor,
  type AgentControlDescriptorApi,
} from "./agent-control-api";

export const desktopAgentControlDescriptorApi: AgentControlDescriptorApi = {
  async get(workspaceId: string, profileId: string): Promise<AgentControlDescriptor> {
    const value = await invoke<unknown>("get_agent_control_descriptor", { workspaceId, profileId });
    return parseAgentControlDescriptor(value);
  },
  async refresh(workspaceId: string, profileId: string): Promise<AgentControlDescriptor> {
    const value = await invoke<unknown>("refresh_agent_control_descriptor", { workspaceId, profileId });
    return parseAgentControlDescriptor(value);
  },
};

export const desktopAgentControlActionsApi: AgentControlActionsApi = {
  async beginAuth(workspaceId: string, profileId: string, methodId: string) {
    const value = await invoke<unknown>("begin_agent_authentication", {
      workspaceId,
      profileId,
      authMethodId: methodId,
    });
    return parseAgentAuthChallenge(value, methodId);
  },
  async openNativeConfig(workspaceId: string, profileId: string, targetId: string) {
    const value = await invoke<unknown>("open_agent_native_configuration", {
      workspaceId,
      profileId,
      targetId,
    });
    return parseNativeConfigOpenReceipt(value, profileId, targetId);
  },
};
