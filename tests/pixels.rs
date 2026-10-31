//! The pixel-digest vectors (CONTRACT.md 2.2), decoded the way rascal decodes: the expected digests
//! in `vectors/pixels/digests.json` are computed by `make.py` from the samples it wrote, not by
//! decoding, so this test checks the recipe against an image library.

use std::path::Path;

use image::DynamicImage;
use sha2::{Digest, Sha256};

fn canonical(image: DynamicImage) -> DynamicImage {
    match image {
        DynamicImage::ImageLuma8(_) | DynamicImage::ImageRgb8(_) | DynamicImage::ImageRgba8(_) => {
            image
        }
        other => DynamicImage::ImageRgb8(other.to_rgb8()),
    }
}

fn pixel_sha256(image: &DynamicImage) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"pixels");
    hasher.update([0u8]);
    hasher.update(image.width().to_le_bytes());
    hasher.update(image.height().to_le_bytes());
    hasher.update(u32::from(image.color().channel_count()).to_le_bytes());
    hasher.update(image.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[test]
fn every_vector_decodes_to_its_recorded_digest_and_file_identity() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("vectors/pixels");
    let digests: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("digests.json")).expect("digests"))
            .expect("digests.json parses");
    let vectors = digests["vectors"].as_array().expect("a vector list");
    assert!(vectors.len() >= 12, "the vector set shrank");
    for vector in vectors {
        let name = vector["file"].as_str().expect("file");
        let bytes = std::fs::read(dir.join(name)).expect("vector file");
        let file_sha256: String = Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(file_sha256, vector["file_sha256"], "{name}: file_sha256");
        let image = canonical(image::load_from_memory(&bytes).expect("decodes"));
        assert_eq!(u64::from(image.width()), vector["width"], "{name}: width");
        assert_eq!(
            u64::from(image.height()),
            vector["height"],
            "{name}: height"
        );
        assert_eq!(
            u64::from(image.color().channel_count()),
            vector["channels"],
            "{name}: channels"
        );
        assert_eq!(
            pixel_sha256(&image),
            vector["pixel_sha256"],
            "{name}: pixel_sha256"
        );
    }
}
