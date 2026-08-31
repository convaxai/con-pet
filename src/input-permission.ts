export const INPUT_MONITORING_AUTO_REQUEST_KEY =
  "con-pet.input-monitoring-auto-requested.v1";

export function shouldAutoRequestInputMonitoring(
  platform: string,
  permissionGranted: boolean,
  requestRecorded: boolean,
): boolean {
  return platform === "macos" && !permissionGranted && !requestRecorded;
}
