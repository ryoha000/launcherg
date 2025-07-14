// ProcTail type definitions for frontend

export interface FileEventData {
  Timestamp: string
  ProcessId: number
  EventType: string
  FilePath: string
  Operation: string
  FileSize: number
  FileAttributes?: string
}

export interface ProcessStartEventData {
  Timestamp: string
  ProcessId: number
  EventType: string
  ProcessName: string
  ParentProcessId: number
  CommandLine: string
  ExecutablePath?: string
}

export interface ProcessEndEventData {
  Timestamp: string
  ProcessId: number
  EventType: string
  ProcessName: string
  ExitCode: number
  ExecutionTime: string
}

export type ProcTailEvent
  = | { File: FileEventData }
    | { ProcessStart: ProcessStartEventData }
    | { ProcessEnd: ProcessEndEventData }

export interface WatchTarget {
  Tag: string
  ProcessId: number
  ProcessName: string
  StartTime: string
  IsRunning: boolean
}

export interface ServiceInfo {
  Status: string
  Version: string
  StartTime: string
  Uptime: string
}

export interface MonitoringInfo {
  EtwSessionActive: boolean
  ActiveTags: number
  ActiveProcesses: number
  TotalEvents: number
}

export interface ResourceInfo {
  MemoryUsageMB: number
  CpuUsagePercent: number
  EstimatedMemoryUsage: number
}

export interface IpcInfo {
  NamedPipeActive: boolean
  ConnectedClients: number
  TotalRequests: number
}

export interface ServiceStatus {
  Service: ServiceInfo
  Monitoring: MonitoringInfo
  Resources: ResourceInfo
  Ipc: IpcInfo
}

export interface HealthCheckResult {
  Status: string
  CheckTime: string
  Details: Record<string, string>
}

export interface AddWatchTargetRequest extends Record<string, unknown> {
  processId: number
  tag: string
}

export interface RemoveWatchTargetRequest extends Record<string, unknown> {
  tag: string
}

export interface GetEventsRequest extends Record<string, unknown> {
  tag: string
  count?: number
  eventType?: string
}

export interface ClearEventsRequest extends Record<string, unknown> {
  tag: string
}

// Form data types
export interface AddTargetForm {
  processId: string
  tag: string
}

export interface GetEventsForm {
  tag: string
  count: string
  eventType: string
}

// Error type for ProcTail operations
export interface ProcTailError {
  message: string
}

// ProcTail Manager types
export interface ProcTailManagerStatus {
  current_version: string | null
  is_running: boolean
  executable_exists: boolean
  update_available: boolean
}

export interface ProcTailVersion {
  version: string
  download_url: string
}
