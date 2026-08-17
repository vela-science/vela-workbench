import { invoke } from "@tauri-apps/api/core";
import type {
  BootstrapDto,
  LaunchKindDto,
  LaunchResultDto,
  PreferencesDto,
  RepositorySnapshotDto,
  VelaBinaryDto,
} from "../contracts/generated/ipc";

export const workbench = {
  bootstrap: () => invoke<BootstrapDto>("bootstrap"),
  selectRepository: () => invoke<RepositorySnapshotDto | null>("select_repository"),
  inspectRepository: (path: string) => invoke<RepositorySnapshotDto>("inspect_repository", { path }),
  selectVelaBinary: () => invoke<VelaBinaryDto | null>("select_vela_binary"),
  clearRecents: () => invoke<PreferencesDto>("clear_recents"),
  launchRepository: (path: string, kind: LaunchKindDto) =>
    invoke<LaunchResultDto>("launch_repository", { path, kind }),
};
