impl serde::Serialize for DmmPackIdsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.store_ids.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("launcherg.packs.DmmPackIdsResponse", len)?;
        if !self.store_ids.is_empty() {
            struct_ser.serialize_field("storeIds", &self.store_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DmmPackIdsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "store_ids",
            "storeIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StoreIds,
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
                            "storeIds" | "store_ids" => Ok(GeneratedField::StoreIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DmmPackIdsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct launcherg.packs.DmmPackIdsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DmmPackIdsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut store_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::StoreIds => {
                            if store_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("storeIds"));
                            }
                            store_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DmmPackIdsResponse {
                    store_ids: store_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("launcherg.packs.DmmPackIdsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetDmmPackIdsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.extension_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("launcherg.packs.GetDmmPackIdsRequest", len)?;
        if !self.extension_id.is_empty() {
            struct_ser.serialize_field("extensionId", &self.extension_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetDmmPackIdsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "extension_id",
            "extensionId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ExtensionId,
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
                            "extensionId" | "extension_id" => Ok(GeneratedField::ExtensionId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetDmmPackIdsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct launcherg.packs.GetDmmPackIdsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetDmmPackIdsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut extension_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ExtensionId => {
                            if extension_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("extensionId"));
                            }
                            extension_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(GetDmmPackIdsRequest {
                    extension_id: extension_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("launcherg.packs.GetDmmPackIdsRequest", FIELDS, GeneratedVisitor)
    }
}
