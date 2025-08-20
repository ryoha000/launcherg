impl serde::Serialize for DlsiteGame {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.id.is_empty() {
            len += 1;
        }
        if !self.category.is_empty() {
            len += 1;
        }
        if self.egs_info.is_some() {
            len += 1;
        }
        if !self.title.is_empty() {
            len += 1;
        }
        if !self.image_url.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("launcherg.sync.DlsiteGame", len)?;
        if !self.id.is_empty() {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if !self.category.is_empty() {
            struct_ser.serialize_field("category", &self.category)?;
        }
        if let Some(v) = self.egs_info.as_ref() {
            struct_ser.serialize_field("egsInfo", v)?;
        }
        if !self.title.is_empty() {
            struct_ser.serialize_field("title", &self.title)?;
        }
        if !self.image_url.is_empty() {
            struct_ser.serialize_field("imageUrl", &self.image_url)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DlsiteGame {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "category",
            "egs_info",
            "egsInfo",
            "title",
            "image_url",
            "imageUrl",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Category,
            EgsInfo,
            Title,
            ImageUrl,
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
                            "id" => Ok(GeneratedField::Id),
                            "category" => Ok(GeneratedField::Category),
                            "egsInfo" | "egs_info" => Ok(GeneratedField::EgsInfo),
                            "title" => Ok(GeneratedField::Title),
                            "imageUrl" | "image_url" => Ok(GeneratedField::ImageUrl),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DlsiteGame;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct launcherg.sync.DlsiteGame")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DlsiteGame, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut category__ = None;
                let mut egs_info__ = None;
                let mut title__ = None;
                let mut image_url__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Category => {
                            if category__.is_some() {
                                return Err(serde::de::Error::duplicate_field("category"));
                            }
                            category__ = Some(map_.next_value()?);
                        }
                        GeneratedField::EgsInfo => {
                            if egs_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("egsInfo"));
                            }
                            egs_info__ = map_.next_value()?;
                        }
                        GeneratedField::Title => {
                            if title__.is_some() {
                                return Err(serde::de::Error::duplicate_field("title"));
                            }
                            title__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ImageUrl => {
                            if image_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("imageUrl"));
                            }
                            image_url__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DlsiteGame {
                    id: id__.unwrap_or_default(),
                    category: category__.unwrap_or_default(),
                    egs_info: egs_info__,
                    title: title__.unwrap_or_default(),
                    image_url: image_url__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("launcherg.sync.DlsiteGame", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DlsiteSyncGamesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.games.is_empty() {
            len += 1;
        }
        if !self.extension_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("launcherg.sync.DlsiteSyncGamesRequest", len)?;
        if !self.games.is_empty() {
            struct_ser.serialize_field("games", &self.games)?;
        }
        if !self.extension_id.is_empty() {
            struct_ser.serialize_field("extensionId", &self.extension_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DlsiteSyncGamesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "games",
            "extension_id",
            "extensionId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Games,
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
                            "games" => Ok(GeneratedField::Games),
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
            type Value = DlsiteSyncGamesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct launcherg.sync.DlsiteSyncGamesRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DlsiteSyncGamesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut games__ = None;
                let mut extension_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Games => {
                            if games__.is_some() {
                                return Err(serde::de::Error::duplicate_field("games"));
                            }
                            games__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ExtensionId => {
                            if extension_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("extensionId"));
                            }
                            extension_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DlsiteSyncGamesRequest {
                    games: games__.unwrap_or_default(),
                    extension_id: extension_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("launcherg.sync.DlsiteSyncGamesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DmmGame {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.id.is_empty() {
            len += 1;
        }
        if !self.category.is_empty() {
            len += 1;
        }
        if !self.subcategory.is_empty() {
            len += 1;
        }
        if self.egs_info.is_some() {
            len += 1;
        }
        if !self.title.is_empty() {
            len += 1;
        }
        if !self.image_url.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("launcherg.sync.DmmGame", len)?;
        if !self.id.is_empty() {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if !self.category.is_empty() {
            struct_ser.serialize_field("category", &self.category)?;
        }
        if !self.subcategory.is_empty() {
            struct_ser.serialize_field("subcategory", &self.subcategory)?;
        }
        if let Some(v) = self.egs_info.as_ref() {
            struct_ser.serialize_field("egsInfo", v)?;
        }
        if !self.title.is_empty() {
            struct_ser.serialize_field("title", &self.title)?;
        }
        if !self.image_url.is_empty() {
            struct_ser.serialize_field("imageUrl", &self.image_url)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DmmGame {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "category",
            "subcategory",
            "egs_info",
            "egsInfo",
            "title",
            "image_url",
            "imageUrl",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Category,
            Subcategory,
            EgsInfo,
            Title,
            ImageUrl,
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
                            "id" => Ok(GeneratedField::Id),
                            "category" => Ok(GeneratedField::Category),
                            "subcategory" => Ok(GeneratedField::Subcategory),
                            "egsInfo" | "egs_info" => Ok(GeneratedField::EgsInfo),
                            "title" => Ok(GeneratedField::Title),
                            "imageUrl" | "image_url" => Ok(GeneratedField::ImageUrl),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DmmGame;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct launcherg.sync.DmmGame")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DmmGame, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut category__ = None;
                let mut subcategory__ = None;
                let mut egs_info__ = None;
                let mut title__ = None;
                let mut image_url__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Category => {
                            if category__.is_some() {
                                return Err(serde::de::Error::duplicate_field("category"));
                            }
                            category__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Subcategory => {
                            if subcategory__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subcategory"));
                            }
                            subcategory__ = Some(map_.next_value()?);
                        }
                        GeneratedField::EgsInfo => {
                            if egs_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("egsInfo"));
                            }
                            egs_info__ = map_.next_value()?;
                        }
                        GeneratedField::Title => {
                            if title__.is_some() {
                                return Err(serde::de::Error::duplicate_field("title"));
                            }
                            title__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ImageUrl => {
                            if image_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("imageUrl"));
                            }
                            image_url__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DmmGame {
                    id: id__.unwrap_or_default(),
                    category: category__.unwrap_or_default(),
                    subcategory: subcategory__.unwrap_or_default(),
                    egs_info: egs_info__,
                    title: title__.unwrap_or_default(),
                    image_url: image_url__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("launcherg.sync.DmmGame", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DmmSyncGamesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.games.is_empty() {
            len += 1;
        }
        if !self.extension_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("launcherg.sync.DmmSyncGamesRequest", len)?;
        if !self.games.is_empty() {
            struct_ser.serialize_field("games", &self.games)?;
        }
        if !self.extension_id.is_empty() {
            struct_ser.serialize_field("extensionId", &self.extension_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DmmSyncGamesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "games",
            "extension_id",
            "extensionId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Games,
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
                            "games" => Ok(GeneratedField::Games),
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
            type Value = DmmSyncGamesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct launcherg.sync.DmmSyncGamesRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DmmSyncGamesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut games__ = None;
                let mut extension_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Games => {
                            if games__.is_some() {
                                return Err(serde::de::Error::duplicate_field("games"));
                            }
                            games__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ExtensionId => {
                            if extension_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("extensionId"));
                            }
                            extension_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DmmSyncGamesRequest {
                    games: games__.unwrap_or_default(),
                    extension_id: extension_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("launcherg.sync.DmmSyncGamesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EgsInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.erogamescape_id != 0 {
            len += 1;
        }
        if !self.gamename.is_empty() {
            len += 1;
        }
        if !self.gamename_ruby.is_empty() {
            len += 1;
        }
        if !self.brandname.is_empty() {
            len += 1;
        }
        if !self.brandname_ruby.is_empty() {
            len += 1;
        }
        if !self.sellday.is_empty() {
            len += 1;
        }
        if self.is_nukige {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("launcherg.sync.EgsInfo", len)?;
        if self.erogamescape_id != 0 {
            struct_ser.serialize_field("erogamescapeId", &self.erogamescape_id)?;
        }
        if !self.gamename.is_empty() {
            struct_ser.serialize_field("gamename", &self.gamename)?;
        }
        if !self.gamename_ruby.is_empty() {
            struct_ser.serialize_field("gamenameRuby", &self.gamename_ruby)?;
        }
        if !self.brandname.is_empty() {
            struct_ser.serialize_field("brandname", &self.brandname)?;
        }
        if !self.brandname_ruby.is_empty() {
            struct_ser.serialize_field("brandnameRuby", &self.brandname_ruby)?;
        }
        if !self.sellday.is_empty() {
            struct_ser.serialize_field("sellday", &self.sellday)?;
        }
        if self.is_nukige {
            struct_ser.serialize_field("isNukige", &self.is_nukige)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EgsInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "erogamescape_id",
            "erogamescapeId",
            "gamename",
            "gamename_ruby",
            "gamenameRuby",
            "brandname",
            "brandname_ruby",
            "brandnameRuby",
            "sellday",
            "is_nukige",
            "isNukige",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ErogamescapeId,
            Gamename,
            GamenameRuby,
            Brandname,
            BrandnameRuby,
            Sellday,
            IsNukige,
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
                            "erogamescapeId" | "erogamescape_id" => Ok(GeneratedField::ErogamescapeId),
                            "gamename" => Ok(GeneratedField::Gamename),
                            "gamenameRuby" | "gamename_ruby" => Ok(GeneratedField::GamenameRuby),
                            "brandname" => Ok(GeneratedField::Brandname),
                            "brandnameRuby" | "brandname_ruby" => Ok(GeneratedField::BrandnameRuby),
                            "sellday" => Ok(GeneratedField::Sellday),
                            "isNukige" | "is_nukige" => Ok(GeneratedField::IsNukige),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EgsInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct launcherg.sync.EgsInfo")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EgsInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut erogamescape_id__ = None;
                let mut gamename__ = None;
                let mut gamename_ruby__ = None;
                let mut brandname__ = None;
                let mut brandname_ruby__ = None;
                let mut sellday__ = None;
                let mut is_nukige__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ErogamescapeId => {
                            if erogamescape_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("erogamescapeId"));
                            }
                            erogamescape_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Gamename => {
                            if gamename__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gamename"));
                            }
                            gamename__ = Some(map_.next_value()?);
                        }
                        GeneratedField::GamenameRuby => {
                            if gamename_ruby__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gamenameRuby"));
                            }
                            gamename_ruby__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Brandname => {
                            if brandname__.is_some() {
                                return Err(serde::de::Error::duplicate_field("brandname"));
                            }
                            brandname__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BrandnameRuby => {
                            if brandname_ruby__.is_some() {
                                return Err(serde::de::Error::duplicate_field("brandnameRuby"));
                            }
                            brandname_ruby__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Sellday => {
                            if sellday__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sellday"));
                            }
                            sellday__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IsNukige => {
                            if is_nukige__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isNukige"));
                            }
                            is_nukige__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(EgsInfo {
                    erogamescape_id: erogamescape_id__.unwrap_or_default(),
                    gamename: gamename__.unwrap_or_default(),
                    gamename_ruby: gamename_ruby__.unwrap_or_default(),
                    brandname: brandname__.unwrap_or_default(),
                    brandname_ruby: brandname_ruby__.unwrap_or_default(),
                    sellday: sellday__.unwrap_or_default(),
                    is_nukige: is_nukige__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("launcherg.sync.EgsInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SyncBatchResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.success_count != 0 {
            len += 1;
        }
        if self.error_count != 0 {
            len += 1;
        }
        if !self.errors.is_empty() {
            len += 1;
        }
        if !self.synced_games.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("launcherg.sync.SyncBatchResult", len)?;
        if self.success_count != 0 {
            struct_ser.serialize_field("successCount", &self.success_count)?;
        }
        if self.error_count != 0 {
            struct_ser.serialize_field("errorCount", &self.error_count)?;
        }
        if !self.errors.is_empty() {
            struct_ser.serialize_field("errors", &self.errors)?;
        }
        if !self.synced_games.is_empty() {
            struct_ser.serialize_field("syncedGames", &self.synced_games)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SyncBatchResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "success_count",
            "successCount",
            "error_count",
            "errorCount",
            "errors",
            "synced_games",
            "syncedGames",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SuccessCount,
            ErrorCount,
            Errors,
            SyncedGames,
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
                            "successCount" | "success_count" => Ok(GeneratedField::SuccessCount),
                            "errorCount" | "error_count" => Ok(GeneratedField::ErrorCount),
                            "errors" => Ok(GeneratedField::Errors),
                            "syncedGames" | "synced_games" => Ok(GeneratedField::SyncedGames),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SyncBatchResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct launcherg.sync.SyncBatchResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SyncBatchResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut success_count__ = None;
                let mut error_count__ = None;
                let mut errors__ = None;
                let mut synced_games__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SuccessCount => {
                            if success_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("successCount"));
                            }
                            success_count__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ErrorCount => {
                            if error_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("errorCount"));
                            }
                            error_count__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Errors => {
                            if errors__.is_some() {
                                return Err(serde::de::Error::duplicate_field("errors"));
                            }
                            errors__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SyncedGames => {
                            if synced_games__.is_some() {
                                return Err(serde::de::Error::duplicate_field("syncedGames"));
                            }
                            synced_games__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SyncBatchResult {
                    success_count: success_count__.unwrap_or_default(),
                    error_count: error_count__.unwrap_or_default(),
                    errors: errors__.unwrap_or_default(),
                    synced_games: synced_games__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("launcherg.sync.SyncBatchResult", FIELDS, GeneratedVisitor)
    }
}
