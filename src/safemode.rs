/// Структура HeaderReferred предоставляет доступ к данным `Header` через ссылки,
/// избегая копирования до явного требования.
pub struct HeaderReferred<'a> {
    pub sig: &'a [u8; 4],  // Ссылка на массив из 4 байт
    pub length: u32,       // Поле `length` извлекается как есть
    pub crc: u32,          // Поле `crc` извлекается как есть
    pub next: &'a [u8; 4], // Ссылка на массив из 4 байт
}

impl<'a> HeaderReferred<'a> {
    /// Безопасный парсинг массива байтов в структуру `HeaderReferred`.
    /// Ожидает, что длина входного массива >= 16 байт.
    pub fn from_bytes(data: &'a [u8]) -> Option<Self> {
        if data.len() < 16 {
            return None; // Недостаточно данных
        }

        let sig = <&[u8; 4]>::try_from(&data[0..4]).ok()?;
        let length = u32::from_le_bytes(data[4..8].try_into().ok()?);
        let crc = u32::from_le_bytes(data[8..12].try_into().ok()?);
        let next = <&[u8; 4]>::try_from(&data[12..16]).ok()?;

        Some(Self {
            sig,
            length,
            crc,
            next,
        })
    }

    /// Возвращает ссылку на сериализованные байты.
    /// Так как мы используем ссылки, само копирование данных не выполняется.
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(16);

        bytes.extend_from_slice(self.sig); // sig
        bytes.extend_from_slice(&self.length.to_le_bytes()); // length в LE
        bytes.extend_from_slice(&self.crc.to_le_bytes()); // crc в LE
        bytes.extend_from_slice(self.next); // next

        bytes
    }

    /// Преобразует `HeaderReferred` в копируемую версию `Header`.
    pub fn to_header(&self) -> Header {
        Header {
            sig: *self.sig,
            length: self.length,
            crc: self.crc,
            next: *self.next,
        }
    }
}

/// Копируемая версия структуры Header для сравнения.
#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub sig: [u8; 4],
    pub length: u32,
    pub crc: u32,
    pub next: [u8; 4],
}

#[test]
fn test() {
    let row_data: [u8; 16] = [
        0x48, 0x45, 0x41, 0x44, // sig: "HEAD" в ASCII
        0x10, 0x00, 0x00, 0x00, // length: 16 (LE формат)
        0xFF, 0xEE, 0xDD, 0xCC, // crc: 0xCCDDEEFF (LE формат)
        0x4E, 0x45, 0x58, 0x54, // next: "NEXT" в ASCII
    ];

    // Парсинг массива байтов в HeaderReferred
    if let Some(header) = HeaderReferred::from_bytes(&row_data) {
        println!("HeaderReferred parsed successfully:");
        println!("  sig: {:?}", std::str::from_utf8(header.sig).unwrap());
        println!("  length: {}", header.length);
        println!("  crc: {:#X}", header.crc);
        println!("  next: {:?}", std::str::from_utf8(header.next).unwrap());

        // Преобразование обратно в массив байтов
        let serialized = header.as_bytes();
        println!("Serialized back to bytes: {:?}", serialized);

        // Преобразование в копируемую структуру Header
        let copied_header = header.to_header();
        println!("Copied Header: {:?}", copied_header);
    } else {
        println!("Failed to parse HeaderReferred from bytes.");
    }
}
