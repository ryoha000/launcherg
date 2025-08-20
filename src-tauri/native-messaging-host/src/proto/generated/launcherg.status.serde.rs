impl serde::Serialize for ExtensionConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.auto_sync {
            len += 1;
        }
        if !self.allowed_domains.is_empty() {
            len += 1;
        }
        if self.sync_interval_minutes != 0 {
            len += 1;
        }
        if self.debug_mode {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("launcherg.status.ExtensionConfig", len)?;
        if self.auto_sync {
            struct_ser.serialize_field("autoSync", &self.auto_sync)?;
        }
        if !self.allowed_domains.is_empty() {
            struct_ser.serialize_field("allowedDomains", &self.allowed_domains)?;
        }
        if self.sync_interval_minutes != 0 {
            struct_ser.serialize_field("syncIntervalMinutes", &self.sync_interval_minutes)?;
        }
        if self.debug_mode {
            struct_ser.serialize_field("debugMode", &self.debug_mode)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExtensionConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "auto_sync",
            "autoSync",
            "allowed_domains",
            "allowedDomains",
            "sync_interval_minutes",
            "syncIntervalMinutes",
            "debug_mode",
            "debugMode",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AutoSync,
            AllowedDomains,
            SyncIntervalMinutes,
            DebugMode,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "autoSync" | "auto_sync" => Ok(GeneratedField::AutoSync),
                            "allowedDomains" | "allowed_domains" => Ok(GeneratedField::AllowedDomains),
                            "syncIntervalMinutes" | "sync_interval_minutes" => Ok(GeneratedField::SyncIntervalMinutes),
                            "debugMode" | "debug_mode" => Ok(GeneratedField::DebugMode),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExtensionConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct launcherg.status.ExtensionConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExtensionConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut auto_sync__ = None;
                let mut allowed_domains__ = None;
                let mut sync_interval_minutes__ = None;
                let mut debug_mode__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AutoSync => {
                            if auto_sync__.is_some() {
                                return Err(serde::de::Error::duplicate_field("autoSync"));
                            }
                            auto_sync__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AllowedDomains => {
                            if allowed_domains__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowedDomains"));
                            }
                            allowed_domains__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SyncIntervalMinutes => {
                            if sync_interval_minutes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("syncIntervalMinutes"));
                            }
                            sync_interval_minutes__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DebugMode => {
                            if debug_mode__.is_some() {
                                return Err(serde::de::Error::duplicate_field("debugMode"));
                            }
                            debug_mode__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ExtensionConfig {
                    auto_sync: auto_sync__.unwrap_or_default(),
                    allowed_domains: allowed_domains__.unwrap_or_default(),
                    sync_interval_minutes: sync_interval_minutes__.unwrap_or_default(),
                    debug_mode: debug_mode__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("launcherg.status.ExtensionConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExtensionConnectionStatus {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "EXTENSION_CONNECTION_STATUS_UNSPECIFIED",
            Self::Connected => "EXTENSION_CONNECTION_STATUS_CONNECTED",
            Self::Connecting => "EXTENSION_CONNECTION_STATUS_CONNECTING",
            Self::HostNotFound => "EXTENSION_CONNECTION_STATUS_HOST_NOT_FOUND",
            Self::HostStartupFailed => "EXTENSION_CONNECTION_STATUS_HOST_STARTUP_FAILED",
            Self::HealthCheckTimeout => "EXTENSION_CONNECTION_STATUS_HEALTH_CHECK_TIMEOUT",
            Self::HealthCheckFailed => "EXTENSION_CONNECTION_STATUS_HEALTH_CHECK_FAILED",
            Self::CommunicationError => "EXTENSION_CONNECTION_STATUS_COMMUNICATION_ERROR",
            Self::ProcessTerminationError => "EXTENSION_CONNECTION_STATUS_PROCESS_TERMINATION_ERROR",
            Self::UnknownError => "EXTENSION_CONNECTION_STATUS_UNKNOWN_ERROR",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ExtensionConnectionStatus {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "EXTENSION_CONNECTION_STATUS_UNSPECIFIED",
            "EXTENSION_CONNECTION_STATUS_CONNECTED",
            "EXTENSION_CONNECTION_STATUS_CONNECTING",
            "EXTENSION_CONNECTION_STATUS_HOST_NOT_FOUND",
            "EXTENSION_CONNECTION_STATUS_HOST_STARTUP_FAILED",
            "EXTENSION_CONNECTION_STATUS_HEALTH_CHECK_TIMEOUT",
            "EXTENSION_CONNECTION_STATUS_HEALTH_CHECK_FAILED",
            "EXTENSION_CONNECTION_STATUS_COMMUNICATION_ERROR",
            "EXTENSION_CONNECTION_STATUS_PROCESS_TERMINATION_ERROR",
            "EXTENSION_CONNECTION_STATUS_UNKNOWN_ERROR",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExtensionConnectionStatus;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "EXTENSION_CONNECTION_STATUS_UNSPECIFIED" => Ok(ExtensionConnectionStatus::Unspecified),
                    "EXTENSION_CONNECTION_STATUS_CONNECTED" => Ok(ExtensionConnectionStatus::Connected),
                    "EXTENSION_CONNECTION_STATUS_CONNECTING" => Ok(ExtensionConnectionStatus::Connecting),
                    "EXTENSION_CONNECTION_STATUS_HOST_NOT_FOUND" => Ok(ExtensionConnectionStatus::HostNotFound),
                    "EXTENSION_CONNECTION_STATUS_HOST_STARTUP_FAILED" => Ok(ExtensionConnectionStatus::HostStartupFailed),
                    "EXTENSION_CONNECTION_STATUS_HEALTH_CHECK_TIMEOUT" => Ok(ExtensionConnectionStatus::HealthCheckTimeout),
                    "EXTENSION_CONNECTION_STATUS_HEALTH_CHECK_FAILED" => Ok(ExtensionConnectionStatus::HealthCheckFailed),
                    "EXTENSION_CONNECTION_STATUS_COMMUNICATION_ERROR" => Ok(ExtensionConnectionStatus::CommunicationError),
                    "EXTENSION_CONNECTION_STATUS_PROCESS_TERMINATION_ERROR" => Ok(ExtensionConnectionStatus::ProcessTerminationError),
                    "EXTENSION_CONNECTION_STATUS_UNKNOWN_ERROR" => Ok(ExtensionConnectionStatus::UnknownError),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for SyncStatus {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.last_sync.is_some() {
            len += 1;
        }
        if self.total_synced != 0 {
            len += 1;
        }
        if !self.connected_extensions.is_empty() {
            len += 1;
        }
        if self.is_running {
            len += 1;
        }
        if self.connection_status != 0 {
            len += 1;
        }
        if !self.error_message.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("launcherg.status.SyncStatus", len)?;
        if let Some(v) = self.last_sync.as_ref() {
            struct_ser.serialize_field("lastSync", v)?;
        }
        if self.total_synced != 0 {
            struct_ser.serialize_field("totalSynced", &self.total_synced)?;
        }
        if !self.connected_extensions.is_empty() {
            struct_ser.serialize_field("connectedExtensions", &self.connected_extensions)?;
        }
        if self.is_running {
            struct_ser.serialize_field("isRunning", &self.is_running)?;
        }
        if self.connection_status != 0 {
            let v = ExtensionConnectionStatus::try_from(self.connection_status)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.connection_status)))?;
            struct_ser.serialize_field("connectionStatus", &v)?;
        }
        if !self.error_message.is_empty() {
            struct_ser.serialize_field("errorMessage", &self.error_message)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SyncStatus {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "last_sync",
            "lastSync",
            "total_synced",
            "totalSynced",
            "connected_extensions",
            "connectedExtensions",
            "is_running",
            "isRunning",
            "connection_status",
            "connectionStatus",
            "error_message",
            "errorMessage",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            LastSync,
            TotalSynced,
            ConnectedExtensions,
            IsRunning,
            ConnectionStatus,
            ErrorMessage,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "lastSync" | "last_sync" => Ok(GeneratedField::LastSync),
                            "totalSynced" | "total_synced" => Ok(GeneratedField::TotalSynced),
                            "connectedExtensions" | "connected_extensions" => Ok(GeneratedField::ConnectedExtensions),
                            "isRunning" | "is_running" => Ok(GeneratedField::IsRunning),
                            "connectionStatus" | "connection_status" => Ok(GeneratedField::ConnectionStatus),
                            "errorMessage" | "error_message" => Ok(GeneratedField::ErrorMessage),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SyncStatus;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct launcherg.status.SyncStatus")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SyncStatus, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut last_sync__ = None;
                let mut total_synced__ = None;
                let mut connected_extensions__ = None;
                let mut is_running__ = None;
                let mut connection_status__ = None;
                let mut error_message__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::LastSync => {
                            if last_sync__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lastSync"));
                            }
                            last_sync__ = map_.next_value()?;
                        }
                        GeneratedField::TotalSynced => {
                            if total_synced__.is_some() {
                                return Err(serde::de::Error::duplicate_field("totalSynced"));
                            }
                            total_synced__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ConnectedExtensions => {
                            if connected_extensions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connectedExtensions"));
                            }
                            connected_extensions__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IsRunning => {
                            if is_running__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isRunning"));
                            }
                            is_running__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ConnectionStatus => {
                            if connection_status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connectionStatus"));
                            }
                            connection_status__ = Some(map_.next_value::<ExtensionConnectionStatus>()? as i32);
                        }
                        GeneratedField::ErrorMessage => {
                            if error_message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("errorMessage"));
                            }
                            error_message__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SyncStatus {
                    last_sync: last_sync__,
                    total_synced: total_synced__.unwrap_or_default(),
                    connected_extensions: connected_extensions__.unwrap_or_default(),
                    is_running: is_running__.unwrap_or_default(),
                    connection_status: connection_status__.unwrap_or_default(),
                    error_message: error_message__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("launcherg.status.SyncStatus", FIELDS, GeneratedVisitor)
    }
}
