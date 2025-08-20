impl serde::Serialize for ConfigUpdateResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.message.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("launcherg.common.ConfigUpdateResult", len)?;
        if !self.message.is_empty() {
            struct_ser.serialize_field("message", &self.message)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConfigUpdateResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "message",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Message,
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
                            "message" => Ok(GeneratedField::Message),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ConfigUpdateResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct launcherg.common.ConfigUpdateResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ConfigUpdateResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut message__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Message => {
                            if message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("message"));
                            }
                            message__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ConfigUpdateResult {
                    message: message__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("launcherg.common.ConfigUpdateResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetStatusRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("launcherg.common.GetStatusRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetStatusRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetStatusRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct launcherg.common.GetStatusRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetStatusRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(GetStatusRequest {
                })
            }
        }
        deserializer.deserialize_struct("launcherg.common.GetStatusRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for HealthCheckRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("launcherg.common.HealthCheckRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for HealthCheckRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = HealthCheckRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct launcherg.common.HealthCheckRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<HealthCheckRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(HealthCheckRequest {
                })
            }
        }
        deserializer.deserialize_struct("launcherg.common.HealthCheckRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for HealthCheckResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.message.is_empty() {
            len += 1;
        }
        if !self.version.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("launcherg.common.HealthCheckResult", len)?;
        if !self.message.is_empty() {
            struct_ser.serialize_field("message", &self.message)?;
        }
        if !self.version.is_empty() {
            struct_ser.serialize_field("version", &self.version)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for HealthCheckResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "message",
            "version",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Message,
            Version,
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
                            "message" => Ok(GeneratedField::Message),
                            "version" => Ok(GeneratedField::Version),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = HealthCheckResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct launcherg.common.HealthCheckResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<HealthCheckResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut message__ = None;
                let mut version__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Message => {
                            if message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("message"));
                            }
                            message__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(HealthCheckResult {
                    message: message__.unwrap_or_default(),
                    version: version__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("launcherg.common.HealthCheckResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NativeMessage {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.timestamp.is_some() {
            len += 1;
        }
        if !self.request_id.is_empty() {
            len += 1;
        }
        if self.message.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("launcherg.common.NativeMessage", len)?;
        if let Some(v) = self.timestamp.as_ref() {
            struct_ser.serialize_field("timestamp", v)?;
        }
        if !self.request_id.is_empty() {
            struct_ser.serialize_field("requestId", &self.request_id)?;
        }
        if let Some(v) = self.message.as_ref() {
            match v {
                native_message::Message::SyncDmmGames(v) => {
                    struct_ser.serialize_field("syncDmmGames", v)?;
                }
                native_message::Message::SyncDlsiteGames(v) => {
                    struct_ser.serialize_field("syncDlsiteGames", v)?;
                }
                native_message::Message::GetStatus(v) => {
                    struct_ser.serialize_field("getStatus", v)?;
                }
                native_message::Message::SetConfig(v) => {
                    struct_ser.serialize_field("setConfig", v)?;
                }
                native_message::Message::HealthCheck(v) => {
                    struct_ser.serialize_field("healthCheck", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NativeMessage {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "timestamp",
            "request_id",
            "requestId",
            "sync_dmm_games",
            "syncDmmGames",
            "sync_dlsite_games",
            "syncDlsiteGames",
            "get_status",
            "getStatus",
            "set_config",
            "setConfig",
            "health_check",
            "healthCheck",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Timestamp,
            RequestId,
            SyncDmmGames,
            SyncDlsiteGames,
            GetStatus,
            SetConfig,
            HealthCheck,
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
                            "timestamp" => Ok(GeneratedField::Timestamp),
                            "requestId" | "request_id" => Ok(GeneratedField::RequestId),
                            "syncDmmGames" | "sync_dmm_games" => Ok(GeneratedField::SyncDmmGames),
                            "syncDlsiteGames" | "sync_dlsite_games" => Ok(GeneratedField::SyncDlsiteGames),
                            "getStatus" | "get_status" => Ok(GeneratedField::GetStatus),
                            "setConfig" | "set_config" => Ok(GeneratedField::SetConfig),
                            "healthCheck" | "health_check" => Ok(GeneratedField::HealthCheck),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NativeMessage;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct launcherg.common.NativeMessage")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NativeMessage, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut timestamp__ = None;
                let mut request_id__ = None;
                let mut message__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = map_.next_value()?;
                        }
                        GeneratedField::RequestId => {
                            if request_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("requestId"));
                            }
                            request_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SyncDmmGames => {
                            if message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("syncDmmGames"));
                            }
                            message__ = map_.next_value::<::std::option::Option<_>>()?.map(native_message::Message::SyncDmmGames)
;
                        }
                        GeneratedField::SyncDlsiteGames => {
                            if message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("syncDlsiteGames"));
                            }
                            message__ = map_.next_value::<::std::option::Option<_>>()?.map(native_message::Message::SyncDlsiteGames)
;
                        }
                        GeneratedField::GetStatus => {
                            if message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("getStatus"));
                            }
                            message__ = map_.next_value::<::std::option::Option<_>>()?.map(native_message::Message::GetStatus)
;
                        }
                        GeneratedField::SetConfig => {
                            if message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("setConfig"));
                            }
                            message__ = map_.next_value::<::std::option::Option<_>>()?.map(native_message::Message::SetConfig)
;
                        }
                        GeneratedField::HealthCheck => {
                            if message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("healthCheck"));
                            }
                            message__ = map_.next_value::<::std::option::Option<_>>()?.map(native_message::Message::HealthCheck)
;
                        }
                    }
                }
                Ok(NativeMessage {
                    timestamp: timestamp__,
                    request_id: request_id__.unwrap_or_default(),
                    message: message__,
                })
            }
        }
        deserializer.deserialize_struct("launcherg.common.NativeMessage", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NativeResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.success {
            len += 1;
        }
        if !self.error.is_empty() {
            len += 1;
        }
        if !self.request_id.is_empty() {
            len += 1;
        }
        if self.response.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("launcherg.common.NativeResponse", len)?;
        if self.success {
            struct_ser.serialize_field("success", &self.success)?;
        }
        if !self.error.is_empty() {
            struct_ser.serialize_field("error", &self.error)?;
        }
        if !self.request_id.is_empty() {
            struct_ser.serialize_field("requestId", &self.request_id)?;
        }
        if let Some(v) = self.response.as_ref() {
            match v {
                native_response::Response::SyncGamesResult(v) => {
                    struct_ser.serialize_field("syncGamesResult", v)?;
                }
                native_response::Response::StatusResult(v) => {
                    struct_ser.serialize_field("statusResult", v)?;
                }
                native_response::Response::ConfigResult(v) => {
                    struct_ser.serialize_field("configResult", v)?;
                }
                native_response::Response::HealthCheckResult(v) => {
                    struct_ser.serialize_field("healthCheckResult", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NativeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "success",
            "error",
            "request_id",
            "requestId",
            "sync_games_result",
            "syncGamesResult",
            "status_result",
            "statusResult",
            "config_result",
            "configResult",
            "health_check_result",
            "healthCheckResult",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Success,
            Error,
            RequestId,
            SyncGamesResult,
            StatusResult,
            ConfigResult,
            HealthCheckResult,
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
                            "success" => Ok(GeneratedField::Success),
                            "error" => Ok(GeneratedField::Error),
                            "requestId" | "request_id" => Ok(GeneratedField::RequestId),
                            "syncGamesResult" | "sync_games_result" => Ok(GeneratedField::SyncGamesResult),
                            "statusResult" | "status_result" => Ok(GeneratedField::StatusResult),
                            "configResult" | "config_result" => Ok(GeneratedField::ConfigResult),
                            "healthCheckResult" | "health_check_result" => Ok(GeneratedField::HealthCheckResult),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NativeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct launcherg.common.NativeResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NativeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut success__ = None;
                let mut error__ = None;
                let mut request_id__ = None;
                let mut response__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Success => {
                            if success__.is_some() {
                                return Err(serde::de::Error::duplicate_field("success"));
                            }
                            success__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Error => {
                            if error__.is_some() {
                                return Err(serde::de::Error::duplicate_field("error"));
                            }
                            error__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RequestId => {
                            if request_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("requestId"));
                            }
                            request_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SyncGamesResult => {
                            if response__.is_some() {
                                return Err(serde::de::Error::duplicate_field("syncGamesResult"));
                            }
                            response__ = map_.next_value::<::std::option::Option<_>>()?.map(native_response::Response::SyncGamesResult)
;
                        }
                        GeneratedField::StatusResult => {
                            if response__.is_some() {
                                return Err(serde::de::Error::duplicate_field("statusResult"));
                            }
                            response__ = map_.next_value::<::std::option::Option<_>>()?.map(native_response::Response::StatusResult)
;
                        }
                        GeneratedField::ConfigResult => {
                            if response__.is_some() {
                                return Err(serde::de::Error::duplicate_field("configResult"));
                            }
                            response__ = map_.next_value::<::std::option::Option<_>>()?.map(native_response::Response::ConfigResult)
;
                        }
                        GeneratedField::HealthCheckResult => {
                            if response__.is_some() {
                                return Err(serde::de::Error::duplicate_field("healthCheckResult"));
                            }
                            response__ = map_.next_value::<::std::option::Option<_>>()?.map(native_response::Response::HealthCheckResult)
;
                        }
                    }
                }
                Ok(NativeResponse {
                    success: success__.unwrap_or_default(),
                    error: error__.unwrap_or_default(),
                    request_id: request_id__.unwrap_or_default(),
                    response: response__,
                })
            }
        }
        deserializer.deserialize_struct("launcherg.common.NativeResponse", FIELDS, GeneratedVisitor)
    }
}
