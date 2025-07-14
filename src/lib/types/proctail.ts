// ProcTail type definitions for frontend

export interface FileEventData {
  timestamp: string
  process_id: number
  event_type: string
  file_path: string
  operation: string
  file_size: number
  file_attributes?: string
}

export interface ProcessStartEventData {
  timestamp: string
  process_id: number
  event_type: string
  process_name: string
  parent_process_id: number
  command_line: string
  executable_path?: string
}

export interface ProcessEndEventData {
  timestamp: string
  process_id: number
  event_type: string
  process_name: string
  exit_code: number
  execution_time: string
}

export type ProcTailEvent
  = | { File: FileEventData }
    | { ProcessStart: ProcessStartEventData }
    | { ProcessEnd: ProcessEndEventData }

export interface WatchTarget {
  tag: string
  process_id: number
  process_name: string
  start_time: string
  is_running: boolean
}

export interface ServiceInfo {
  status: string
  version: string
  start_time: string
  uptime: string
}

export interface MonitoringInfo {
  etw_session_active: boolean
  active_tags: number
  active_processes: number
  total_events: number
}

export interface ResourceInfo {
  memory_usage_mb: number
  cpu_usage_percent: number
  estimated_memory_usage: number
}

export interface IpcInfo {
  named_pipe_active: boolean
  connected_clients: number
  total_requests: number
}

export interface ServiceStatus {
  service: ServiceInfo
  monitoring: MonitoringInfo
  resources: ResourceInfo
  ipc: IpcInfo
}

export interface HealthCheckResult {
  status: string
  check_time: string
  details: Record<string, string>
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
