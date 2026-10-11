use icu_properties::CodePointMapData;
use icu_properties::props::GeneralCategory;
use steplet_core::display::{is_invisible, is_space_like};

#[test]
fn space_like_is_exactly_category_zs_without_the_space() {
    let gc = CodePointMapData::<GeneralCategory>::new();
    for cp in 0..=0x10_ffffu32 {
        let Some(c) = char::from_u32(cp) else { continue };
        let zs = gc.get(c) == GeneralCategory::SpaceSeparator && c != ' ';
        assert_eq!(is_space_like(c), zs, "U+{cp:04X}");
        if zs {
            assert!(is_invisible(c), "U+{cp:04X} gets a badge too");
        }
    }
}
