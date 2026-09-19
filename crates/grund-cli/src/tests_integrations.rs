/// Test module: the `grund integrations` command — its argv, its descriptors and
/// the form its `--write` reports (§FS-integrations). The cases moved here with
/// the command when it left the engine for the frontend that owns its rendering
/// (§AR-bindings.3, §DA-engine-renders-nothing); what the engine still answers —
/// the client set, the splices, the detection probe — is pinned beside it, in
/// `crates/grund-core/src/writers/tests_integrations*.rs`.
#[cfg(test)]
mod tests_integrations {
    use super::*;

    /// §FS-integrations.3.4: iTerm2 keeps its rules in a binary plist, so there is
    /// nothing to splice and nothing to read back. It must never claim installed —
    /// a guess there is worse than reporting nothing — and the detection plan has
    /// to say *why*, so a caller can tell "not installed" from "not knowable".
    #[test]
    fn iterm2_is_a_manual_client_that_never_claims_installed() {
        assert!(matches!(
            IntegrationClient::Iterm2.install_kind(),
            InstallKind::Manual
        ));
        assert!(!integration_is_current(IntegrationClient::Iterm2));
        let descriptor = client_descriptor_json(IntegrationClient::Iterm2);
        assert!(descriptor.contains("\"install_kind\":\"manual\""));
        // It still uses the shared resolver, so it counts as a terminal client
        // and its printed artifact carries grund-open.
        assert!(IntegrationClient::Iterm2.is_terminal());
        // The rule it prints must carry the same matcher the other clients use,
        // or a citation clickable in kitty would be inert in iTerm2.
        let snippet = IntegrationClient::Iterm2
            .snippet()
            .expect("iterm2 artifact");
        assert!(snippet.contains("[A-Z][A-Z0-9]*-[a-z0-9][a-z0-9-]*"));
        assert!(snippet.contains("grund-open \\0"));
    }

    // §FS-integrations.2.1: detection closes on the preview line and the setup
    // guide, in both the detected and the nothing-detected form — the guide
    // carries the prerequisites and manual steps `--write` cannot perform.
    #[test]
    fn detection_names_the_setup_guide() {
        assert!(SETUP_GUIDE_URL.starts_with("https://"));
        assert!(SETUP_GUIDE_URL.ends_with("docs/user-facing/clickable-citations.md"));
    }

    // §FS-integrations.1.1 / §FS-integrations.4.3: an explicit conversation
    // preference is a complete clientless write target.
    #[test]
    fn integrations_accepts_preference_only_write() {
        let args = ["--write", "--conversation", "link"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let invocation = parse_integrations_args(&args).expect("parse preference-only write");
        assert!(invocation.client.is_none());
        assert!(invocation.write);
        assert_eq!(invocation.conversation, Some(ConversationRendering::Link));
    }

    // §FS-integrations.1.2: `--conversation-target` alone is also a complete
    // clientless write target, and an unknown value is a CLI error listing the
    // accepted set — a value the caller typed, not a stale line in a file.
    #[test]
    fn integrations_accepts_target_only_write() {
        let args = ["--write", "--conversation-target", "vscodium"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let invocation = parse_integrations_args(&args).expect("parse target-only write");
        assert!(invocation.client.is_none());
        assert!(invocation.write);
        assert_eq!(invocation.conversation, None);
        assert_eq!(
            invocation.conversation_target,
            Some(ConversationTarget::Vscodium)
        );

        let joined = ["--write", "--conversation-target=web"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        assert_eq!(
            parse_integrations_args(&joined)
                .expect("parse joined form")
                .conversation_target,
            Some(ConversationTarget::Web)
        );

        for rejected in [
            vec!["--write", "--conversation-target", "emacs"],
            vec!["--conversation-target", "file"],
        ] {
            let args = rejected.into_iter().map(str::to_string).collect::<Vec<_>>();
            assert!(
                parse_integrations_args(&args).is_err(),
                "must be rejected: {args:?}"
            );
        }
    }

    // §FS-integrations.4.4.4: the report names the form each agent received, and
    // why when it is not the one asked for — unreported, an override, a gate
    // downgrade, and an unread key look identical from the outside.
    #[test]
    fn effective_form_describes_override_and_gate() {
        let plain = EffectiveForm {
            rendering: ConversationRendering::Plain,
            target: ConversationTarget::Path,
            requested: ConversationTarget::Vscodium,
            overridden: false,
        };
        assert_eq!(plain.describe(), "plain");

        let taken = EffectiveForm {
            rendering: ConversationRendering::Link,
            target: ConversationTarget::Vscodium,
            requested: ConversationTarget::Vscodium,
            overridden: false,
        };
        assert_eq!(taken.describe(), "link \u{2192} vscodium");

        let overridden = EffectiveForm {
            rendering: ConversationRendering::Link,
            target: ConversationTarget::Web,
            requested: ConversationTarget::Web,
            overridden: true,
        };
        assert_eq!(overridden.describe(), "link \u{2192} web; agent override");

        let gated = EffectiveForm {
            rendering: ConversationRendering::Link,
            target: ConversationTarget::Path,
            requested: ConversationTarget::Vscodium,
            overridden: false,
        };
        assert_eq!(
            gated.describe(),
            "link \u{2192} path; vscodium unverified here"
        );
    }

    // §FS-integrations.1.2 / §FS-integrations.6.2: `--agent` scopes
    // `--conversation-target` and nothing else.
    #[test]
    fn agent_flag_requires_write_and_a_target() {
        let ok = [
            "--write",
            "--agent",
            "codex",
            "--conversation-target",
            "web",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        let invocation = parse_integrations_args(&ok).expect("parse scoped write");
        assert_eq!(invocation.agent, Some("codex"));
        assert_eq!(
            invocation.conversation_target,
            Some(ConversationTarget::Web)
        );

        for rejected in [
            // no --write
            vec!["--agent", "codex", "--conversation-target", "web"],
            // nothing to scope
            vec!["--write", "--agent", "codex"],
            // unknown agent
            vec!["--write", "--agent", "codx", "--conversation-target", "web"],
        ] {
            let args = rejected.into_iter().map(str::to_string).collect::<Vec<_>>();
            assert!(
                parse_integrations_args(&args).is_err(),
                "must be rejected: {args:?}"
            );
        }
    }

    /// §FS-integrations.5: the machine detection plan distinguishes ambient
    /// detection from actual installation state, and carries each client's
    /// `install_kind` so a manual client's permanent `installed: false` reads as
    /// "not knowable" rather than "not installed" (§FS-integrations.3.4.2).
    #[test]
    fn integrations_detection_json_reports_installed_state() {
        let json = detection_plan_json(&[IntegrationClient::Wezterm]);
        assert!(json.contains("\"client\":\"wezterm\",\"detected\":true,\"installed\":"));
        assert!(json.contains("\"client\":\"kitty\",\"detected\":false,\"installed\":"));
        assert!(json.contains("\"install_kind\":\"block\""));
        assert!(json.contains("\"install_kind\":\"extension\""));
        assert!(json.contains(
            "\"client\":\"iterm2\",\"detected\":false,\"installed\":false,\"install_kind\":\"manual\""
        ));
    }
}
