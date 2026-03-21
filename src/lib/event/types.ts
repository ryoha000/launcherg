import type {
  PubSubEvent,
} from '../typeshare/pubsub'

export type EventName = PubSubEvent['type']

type ExtractPayload<T extends EventName> = Extract<PubSubEvent, { type: T }> extends {
  payload: infer P
}
  ? P
  : never

export type EventPayloadMap = {
  [K in EventName]: ExtractPayload<K>
}

export type TypedEventHandler<T extends EventName> = (payload: EventPayloadMap[T]) => void

export type {
  AppSignalEventPayload,
  AppSignalPayload,
  AppSignalSourcePayload,
  DedupResultPayload,
  EnrichResultPayload,
  ImageQueueItemErrorPayload,
  ImageQueueItemPayload,
  ImageQueueWorkerStatusPayload,
  ProgressLivePayload,
  ProgressPayload,
  PubSubEvent,
  ScanCandidateDiscoveredPayload,
  ScanExploreFinishedPayload,
  ScanLogPayload,
  ScanPhaseTimingPayload,
  ScanProgressPayload,
  ScanSummaryPayload,
} from '../typeshare/pubsub'
