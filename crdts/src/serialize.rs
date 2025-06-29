pub trait Serialize {
    fn to_bytes(&self) -> Vec<u8>;
}

impl Serialize for String {
    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl Serialize for Vec<u8> {
    fn to_bytes(&self) -> Vec<u8> {
        self.clone()
    }
}

impl Serialize for u64 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Serialize for u32 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Serialize for u16 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Serialize for u8 {
    fn to_bytes(&self) -> Vec<u8> {
        vec![*self]
    }
}

impl Serialize for bool {
    fn to_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

impl Serialize for i64 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Serialize for i32 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Serialize for i16 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Serialize for i8 {
    fn to_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

impl Serialize for &str {
    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl Serialize for char {
    fn to_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

impl<T: Serialize> Serialize for (T, T) {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.0.to_bytes());
        bytes.extend(self.1.to_bytes());
        bytes
    }
}

impl<T: Serialize> Serialize for &T {
    fn to_bytes(&self) -> Vec<u8> {
        (*self).to_bytes()
    }
}