//! Optional Basic auth from `OCI_REGISTRY_PASSWORD`.

use std::env;

#[derive(Debug, Clone)]
pub struct Secrets {
    /// When set, every request must present this password (username ignored).
    pub password: Option<String>,
}

impl Secrets {
    pub fn from_env() -> Self {
        Self {
            password: env::var("OCI_REGISTRY_PASSWORD")
                .ok()
                .filter(|s| !s.is_empty()),
        }
    }

    pub fn authorize_basic(&self, authorization: Option<&str>) -> Result<(), AuthError> {
        let Some(expected) = self.password.as_deref() else {
            return Ok(());
        };
        let Some(header) = authorization else {
            return Err(AuthError::Missing);
        };
        let Some(token) = header.strip_prefix("Basic ") else {
            return Err(AuthError::Missing);
        };
        let Ok(decoded) = b64_decode(token.trim()) else {
            return Err(AuthError::Invalid);
        };
        let Ok(pair) = String::from_utf8(decoded) else {
            return Err(AuthError::Invalid);
        };
        let password = pair
            .split_once(':')
            .map(|(_, p)| p)
            .unwrap_or(pair.as_str());
        if password == expected {
            Ok(())
        } else {
            Err(AuthError::Denied)
        }
    }
}

#[derive(Debug)]
pub enum AuthError {
    Missing,
    Invalid,
    Denied,
}

fn b64_decode(input: &str) -> Result<Vec<u8>, ()> {
    const T: [i8; 128] = {
        let mut t = [-1i8; 128];
        let mut i = 0;
        while i < 26 {
            t[b'A' as usize + i] = i as i8;
            t[b'a' as usize + i] = (26 + i) as i8;
            i += 1;
        }
        i = 0;
        while i < 10 {
            t[b'0' as usize + i] = (52 + i) as i8;
            i += 1;
        }
        t[b'+' as usize] = 62;
        t[b'/' as usize] = 63;
        t
    };

    let bytes = input.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return Err(());
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks_exact(4) {
        let mut vals = [0u8; 4];
        let mut pad = 0usize;
        for (i, &b) in chunk.iter().enumerate() {
            if b == b'=' {
                if i < 2 {
                    return Err(());
                }
                pad += 1;
                vals[i] = 0;
                continue;
            }
            if b >= 128 || T[b as usize] < 0 {
                return Err(());
            }
            vals[i] = T[b as usize] as u8;
        }
        out.push((vals[0] << 2) | (vals[1] >> 4));
        if pad < 2 {
            out.push((vals[1] << 4) | (vals[2] >> 2));
        }
        if pad < 1 {
            out.push((vals[2] << 6) | vals[3]);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_matching_password() {
        let secrets = Secrets {
            password: Some("sekret".into()),
        };
        assert!(secrets.authorize_basic(Some("Basic eDpzZWtyZXQ=")).is_ok());
    }

    #[test]
    fn rejects_wrong_password() {
        let secrets = Secrets {
            password: Some("sekret".into()),
        };
        assert!(secrets.authorize_basic(Some("Basic eDp3cm9uZw==")).is_err());
    }
}
