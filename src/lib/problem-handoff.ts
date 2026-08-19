import { getCurrent, onOpenUrl } from "@tauri-apps/plugin-deep-link";

export async function observeProblemHandoffUrls(
  accept: (url: string) => void,
): Promise<() => void> {
  const unlisten = await onOpenUrl((urls) => urls.forEach(accept));
  const current = await getCurrent();
  current?.forEach(accept);
  return unlisten;
}
