// Re-export all protobuf types for convenience

// Extension Internal types
export * from './extension_internal/messages_pb'
// Native Messaging types (with namespace prefixes to avoid conflicts)
export * as NativeMessaging from './native_messaging/common_pb'
export * as NativeStatus from './native_messaging/status_pb'

export * as NativeSync from './native_messaging/sync_pb'
