export type FrameworkConfidence = "high" | "medium" | "low";

export type FrameworkDetectionSource =
  | "command"
  | "package_json"
  | "config_file"
  | "http_probe"
  | "unknown";

export interface FrameworkDetection {
  name: string;
  confidence: FrameworkConfidence;
  source: FrameworkDetectionSource;
}
