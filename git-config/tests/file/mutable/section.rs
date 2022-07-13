mod push {
    use git_config::parse::section::Key;
    use std::borrow::Cow;
    use std::convert::TryFrom;

    #[test]
    fn whitespace_is_derived_from_whitespace_before_first_value() -> crate::Result {
        for (input, expected) in [
            ("[a]\n\t\tb = c", Some("\t\t".into())),
            ("[a]\nb = c", None),
            ("[a]", Some("\t".into())),
            ("[a]\t\tb = c", Some("\t\t".into())),
            ("[a]\n\t\t  \n    \t    b = c", Some("    \t    ".into())),
        ] {
            let mut config: git_config::File = input.parse()?;
            assert_eq!(
                config.section_mut("a", None)?.leading_whitespace(),
                expected,
                "{:?} should find {:?} as whitespace",
                input,
                expected
            )
        }
        Ok(())
    }

    #[test]
    fn push_splits_values_into_events() {
        let mut config = git_config::File::default();
        let mut section = config.new_section("core", None).unwrap();
        section.push(Key::try_from("value").unwrap(), Cow::Borrowed("none".into()));
        assert_eq!(config.to_bstring(), "[core]\n\tvalue=none\n");
    }
}
