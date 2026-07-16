import type { PermissionStatus } from "../types/voxta";

interface Props {
  permissions: PermissionStatus;
  onOpen: (permission: "microphone" | "accessibility") => void;
}

function readableStatus(value: string) {
  switch (value) {
    case "granted":
      return "Allowed";
    case "denied":
      return "Needs attention";
    case "notDetermined":
      return "Not requested";
    case "notRequired":
      return "Not required";
    default:
      return "Unknown";
  }
}

export function PermissionRows({ permissions, onOpen }: Props) {
  return (
    <div className="panel">
      <h2>Permissions</h2>
      <div className="permission-row">
        <div>
          <strong>Microphone</strong>
          <span>{readableStatus(permissions.microphone)}</span>
        </div>
        <button type="button" className="secondary" onClick={() => onOpen("microphone")}>
          Open
        </button>
      </div>
      <div className="permission-row">
        <div>
          <strong>Accessibility</strong>
          <span>{readableStatus(permissions.accessibility)}</span>
        </div>
        <button type="button" className="secondary" onClick={() => onOpen("accessibility")}>
          Open
        </button>
      </div>
    </div>
  );
}
