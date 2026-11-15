//! HKDF-SHA256 (RFC 5869) und die Schlüsselableitungen aus Kap. 4.2.
//!
//! Die IKM-Teile werden in dieser Reihenfolge konkateniert. Eine leere `salt` bedeutet
//! „keine Salt“ (RFC 5869: Nullsalt der Hash-Länge).

use crate::domain;
use crate::error::CryptoError;
use crate::key::{DataKey, DocumentKey, IndexSecret, MasterKey, SubjectKey, KEY_LEN};
use crate::mac::hmac_sha256;
use hkdf::Hkdf;
use sha2::Sha256;
use unicode_normalization::UnicodeNormalization;
use zeroize::Zeroizing;

/// Betroffenen-Kennung (`SubjectTag`, Spec 2.4). 16 Byte, nicht geheim.
pub type SubjectTag = [u8; 16];

/// Höchstzahl der Betroffenen einer Version. Die Formel kodiert `n` als `u8`.
/// Anhang A nennt bis 256; der Wert 256 ist im Format nicht darstellbar (siehe ADR-Hinweis).
pub const MAX_SUBJECTS_PER_VERSION: usize = 255;

/// HKDF-SHA256 über die konkatenierten `ikm`-Teile, mit `salt` und `info`.
///
/// # Errors
/// [`CryptoError::Internal`], wenn die Ausgabelänge nicht erreichbar ist (nicht erwartet).
pub fn hkdf_sha256(
    ikm: &[&[u8]],
    salt: &[u8],
    info: &[u8],
) -> Result<Zeroizing<[u8; KEY_LEN]>, CryptoError> {
    let total: usize = ikm.iter().map(|p| p.len()).sum();
    let mut joined = Zeroizing::new(Vec::with_capacity(total));
    for part in ikm {
        joined.extend_from_slice(part);
    }
    let salt = if salt.is_empty() { None } else { Some(salt) };
    let hk = Hkdf::<Sha256>::new(salt, &joined);
    let mut out = Zeroizing::new([0_u8; KEY_LEN]);
    hk.expand(info, out.as_mut_slice())
        .map_err(|_| CryptoError::Internal)?;
    Ok(out)
}

/// Datenschlüssel DEK(v) (Spec 4.2):
/// `HKDF(DocumentKey ‖ SubjectKey(s1) ‖ … ‖ SubjectKey(sn), salt = VerRef, info = "acta/dek/v1" ‖ n)`.
///
/// Die Betroffenen werden intern aufsteigend nach `SubjectTag` sortiert. Die Eingabereihenfolge
/// ist damit ohne Wirkung. Doppelte Tags sind ein Fehler.
///
/// # Errors
/// [`CryptoError::TooManySubjects`] bei mehr als [`MAX_SUBJECTS_PER_VERSION`],
/// [`CryptoError::DuplicateSubject`] bei doppelten Tags.
pub fn derive_dek(
    doc: &DocumentKey,
    ver_ref: &[u8; 16],
    subjects: &[(SubjectTag, &SubjectKey)],
) -> Result<DataKey, CryptoError> {
    if subjects.len() > MAX_SUBJECTS_PER_VERSION {
        return Err(CryptoError::TooManySubjects);
    }
    let n = u8::try_from(subjects.len()).map_err(|_| CryptoError::TooManySubjects)?;

    let mut sorted: Vec<&(SubjectTag, &SubjectKey)> = subjects.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    if sorted.windows(2).any(|w| matches!(w, [x, y] if x.0 == y.0)) {
        return Err(CryptoError::DuplicateSubject);
    }

    let mut ikm: Vec<&[u8]> = Vec::with_capacity(sorted.len() + 1);
    ikm.push(doc.expose());
    for entry in &sorted {
        ikm.push(entry.1.expose());
    }
    let mut info = Vec::with_capacity(domain::DEK.len() + 1);
    info.extend_from_slice(domain::DEK);
    info.push(n);

    let okm = hkdf_sha256(&ikm, ver_ref, &info)?;
    Ok(DataKey::from_derived(okm))
}

/// Dokumentkopf-Schlüssel HK(v) = `HKDF(DocumentKey(v), salt = VerRef(v), info = "acta/hdr/v1")`.
///
/// # Errors
/// [`CryptoError::Internal`] nur bei internem Fehler der HKDF-Ausgabe.
pub fn derive_header_key(doc: &DocumentKey, ver_ref: &[u8; 16]) -> Result<DataKey, CryptoError> {
    let okm = hkdf_sha256(&[doc.expose()], ver_ref, domain::HDR)?;
    Ok(DataKey::from_derived(okm))
}

/// Verzeichnisschlüssel DirKey(s) = `HKDF(SubjectKey(s), info = "acta/dir/v1")`.
///
/// # Errors
/// [`CryptoError::Internal`] nur bei internem Fehler der HKDF-Ausgabe.
pub fn derive_dir_key(subject: &SubjectKey) -> Result<DataKey, CryptoError> {
    let okm = hkdf_sha256(&[subject.expose()], &[], domain::DIR)?;
    Ok(DataKey::from_derived(okm))
}

/// Clusterkennung ClusterId(s) = `HKDF(SubjectKey(s), info = "acta/cluster/v1")[..16]`.
/// Sie wird nicht gespeichert, sondern nur mit dem `SubjectKey` berechnet (Spec 4.2).
///
/// # Errors
/// [`CryptoError::Internal`] nur bei internem Fehler der HKDF-Ausgabe.
pub fn derive_cluster_id(subject: &SubjectKey) -> Result<[u8; 16], CryptoError> {
    let okm = hkdf_sha256(&[subject.expose()], &[], domain::CLUSTER)?;
    let head = okm.get(..16).ok_or(CryptoError::Internal)?;
    <[u8; 16]>::try_from(head).map_err(|_| CryptoError::Internal)
}

/// Tresorschlüssel VK(t,g) = `HKDF(M_g, salt = TenantId, info = "acta/vault/v1" ‖ g u32)` (Spec 4.2).
///
/// # Errors
/// [`CryptoError::Internal`] nur bei internem Fehler der HKDF-Ausgabe.
pub fn derive_vault_key(
    master: &MasterKey,
    tenant: &[u8; 16],
    generation: u32,
) -> Result<DataKey, CryptoError> {
    let mut info = Vec::with_capacity(domain::VAULT.len() + 4);
    info.extend_from_slice(domain::VAULT);
    info.extend_from_slice(&generation.to_be_bytes());
    let okm = hkdf_sha256(&[master.expose()], tenant, &info)?;
    Ok(DataKey::from_derived(okm))
}

/// Tag-Schlüssel K_tag(t) = `HKDF(k_idx, salt = TenantId, info = "acta/tag-key/v1")` (Spec 2.4).
///
/// # Errors
/// [`CryptoError::Internal`] nur bei internem Fehler der HKDF-Ausgabe.
pub fn derive_tag_key(index: &IndexSecret, tenant: &[u8; 16]) -> Result<DataKey, CryptoError> {
    let okm = hkdf_sha256(&[index.expose()], tenant, domain::TAG_KEY)?;
    Ok(DataKey::from_derived(okm))
}

/// Betroffenen-Tag `HMAC-SHA256(K_tag(t), "acta/tag/v1" ‖ kind ‖ NFC(ext_id))[..16]` (Spec 2.4).
///
/// `ext_id` trägt seinen Namensraum (etwa `az:`, `pnr:`, `oidc:<iss>|<sub>`). Die Zeichenkette
/// wird vor dem Hashen in NFC normalisiert, damit gleiche Kennungen gleiche Tags ergeben.
///
/// # Errors
/// [`CryptoError::Internal`] nur bei internem Fehler der HMAC-Berechnung.
pub fn derive_subject_tag(
    index: &IndexSecret,
    tenant: &[u8; 16],
    kind: u8,
    ext_id: &str,
) -> Result<SubjectTag, CryptoError> {
    let key = derive_tag_key(index, tenant)?;
    let nfc: String = ext_id.nfc().collect();
    let full = hmac_sha256(key.expose(), &[domain::TAG, &[kind], nfc.as_bytes()])?;
    let head = full.get(..16).ok_or(CryptoError::Internal)?;
    <[u8; 16]>::try_from(head).map_err(|_| CryptoError::Internal)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    // Festwerte aus einer unabhängigen Python-Referenz (HKDF per HMAC-SHA256, siehe Spec 16.1).
    fn fixtures() -> (DocumentKey, [u8; 16], SubjectKey, SubjectKey) {
        (
            DocumentKey::from_bytes([0xAA; 32]),
            [0xBB; 16],
            SubjectKey::from_bytes([0x11; 32]),
            SubjectKey::from_bytes([0x22; 32]),
        )
    }

    #[test]
    fn dek_with_two_subjects_matches_reference() -> Result<(), CryptoError> {
        let (doc, ver, a, b) = fixtures();
        let dek = derive_dek(&doc, &ver, &[([0x01; 16], &a), ([0x02; 16], &b)])?;
        assert_eq!(
            hex(dek.expose()),
            "b7179429b30a477678c20625652d753967063c5ed72ea8f975db6e90b8cdf9f7"
        );
        Ok(())
    }

    #[test]
    fn dek_without_subjects_matches_reference() -> Result<(), CryptoError> {
        let (doc, ver, _, _) = fixtures();
        let dek = derive_dek(&doc, &ver, &[])?;
        assert_eq!(
            hex(dek.expose()),
            "479452172b726bc1a034a435ee563e4974ea9e39267a52e312a8d66fd6acb1e0"
        );
        Ok(())
    }

    #[test]
    fn dek_ignores_input_order_of_subjects() -> Result<(), CryptoError> {
        let (doc, ver, a, b) = fixtures();
        let forward = derive_dek(&doc, &ver, &[([0x01; 16], &a), ([0x02; 16], &b)])?;
        let reverse = derive_dek(&doc, &ver, &[([0x02; 16], &b), ([0x01; 16], &a)])?;
        assert_eq!(forward.expose(), reverse.expose());
        Ok(())
    }

    #[test]
    fn dek_rejects_duplicate_tags_and_too_many_subjects() -> Result<(), CryptoError> {
        let (doc, ver, a, b) = fixtures();
        let dup = derive_dek(&doc, &ver, &[([0x01; 16], &a), ([0x01; 16], &b)]);
        assert!(matches!(dup, Err(CryptoError::DuplicateSubject)));

        let many: Vec<([u8; 16], &SubjectKey)> = (0..256_u16)
            .map(|i| {
                let mut t = [0_u8; 16];
                if let Some(head) = t.get_mut(..2) {
                    head.copy_from_slice(&i.to_be_bytes());
                }
                (t, &a)
            })
            .collect();
        let res = derive_dek(&doc, &ver, &many);
        assert!(matches!(res, Err(CryptoError::TooManySubjects)));
        Ok(())
    }

    #[test]
    fn header_directory_and_cluster_match_reference() -> Result<(), CryptoError> {
        let (doc, ver, a, _) = fixtures();
        assert_eq!(
            hex(derive_header_key(&doc, &ver)?.expose()),
            "285606c358d0adae1650694513f8f250ff3671448266a38fd9656a30108fd08f"
        );
        assert_eq!(
            hex(derive_dir_key(&a)?.expose()),
            "c9d4c2b8a09cd4779834f3e20ea7b6e1392c0185cf429a91bf2cc58cdb26c8d4"
        );
        assert_eq!(
            hex(&derive_cluster_id(&a)?),
            "8f829f8b66517ad6042739a6f40669cc"
        );
        Ok(())
    }

    #[test]
    fn rfc5869_case1_hkdf_matches() -> Result<(), CryptoError> {
        let ikm = [0x0b_u8; 22];
        let salt: Vec<u8> = (0x00..=0x0c).collect();
        let info: Vec<u8> = (0xf0..=0xf9).collect();
        let okm = hkdf_sha256(&[&ikm], &salt, &info)?;
        assert_eq!(
            hex(okm.as_slice()),
            // Die ersten 32 Byte der RFC-5869-Ausgabe (L = 42); HKDF ist präfixstabil.
            "3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf"
        );
        Ok(())
    }
}

#[cfg(test)]
mod derive_reference_tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn vault_key_matches_reference_per_generation() -> Result<(), CryptoError> {
        let master = MasterKey::from_bytes([0x33; 32]);
        let tenant = [0x44; 16];
        assert_eq!(
            hex(derive_vault_key(&master, &tenant, 0)?.expose()),
            "2ff1ed67650a6b7cc3dd73e3c1f2df3406de95b0153db1e96457f6a20e3c3d5c"
        );
        assert_eq!(
            hex(derive_vault_key(&master, &tenant, 7)?.expose()),
            "23d3e4b774c5519fb68fe4f5c424a905abc4f390312ecbb5346e04b1c36bb64c"
        );
        Ok(())
    }

    #[test]
    fn tag_key_matches_reference() -> Result<(), CryptoError> {
        let index = IndexSecret::from_bytes([0x55; 32]);
        let tenant = [0x44; 16];
        assert_eq!(
            hex(derive_tag_key(&index, &tenant)?.expose()),
            "fd025f739481f6bc6220f15e3e862f579a5b239953f016049ec0a6770c2c70e5"
        );
        Ok(())
    }
}
