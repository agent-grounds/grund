//! The passes that read a form again before it is offered (§FS-rules.3.5.4.3),
//! driven by readers written for each case so that a pass the parser never
//! takes today is still pinned.

use super::{Form, PRODUCTIONS, Refusal, refuse, settle};

/// Settle `form` under `answer`, a reader of the form it is given, and count
/// the passes it was read in.
fn settled(
    form: &str,
    mut answer: impl FnMut(&str, bool) -> Option<Refusal>,
) -> (Result<(String, bool), bool>, usize) {
    let mut passes = 0;
    let settled = settle(form.into(), false, |form, enabled| {
        passes += 1;
        answer(form, enabled)
    });
    (settled, passes)
}

/// A form is offered as the pass that accepts it leaves it.
#[test]
fn a_form_is_offered_once_a_pass_accepts_it() {
    let (settled, passes) = settled("Each FS must cite at least 1 FS.", |form, _| {
        form.contains(" 1 ").then(|| {
            refuse(
                "numeric",
                Form::One(form.replace("at least 1", "at least one")),
            )
        })
    });
    assert_eq!(
        settled,
        Ok(("Each FS must cite at least one FS.".into(), false))
    );
    assert_eq!(passes, 2, "one pass to replace, one to accept");
}

/// The passes end within the productions a sentence has, even where every pass
/// would rewrite the sentence again.
#[test]
fn the_passes_are_bounded_by_the_productions_of_a_sentence() {
    let (settled, passes) = settled("Each FS must cite FS.", |form, _| {
        Some(refuse("never accepted", Form::One(format!("{form}."))))
    });
    assert_eq!(settled, Err(false), "an unbounded rewrite offers no form");
    assert_eq!(passes, PRODUCTIONS);
}

/// A pass that would leave the sentence as it was ends with no form.
#[test]
fn a_pass_that_changes_nothing_ends_with_no_form() {
    let (settled, passes) = settled("Each FS must cite FS.", |form, _| {
        Some(refuse("refused again", Form::One(form.into())))
    });
    assert_eq!(settled, Err(false));
    assert_eq!(passes, 1);
}

/// Turning named sections on is a change of its own, so a form kept as typed
/// for that reason is read again with them on (§FS-rules.3.5.2).
#[test]
fn a_pass_that_only_turns_named_sections_on_reads_the_form_again() {
    let (settled, passes) = settled("FS-login.requirements must cite REQ.", |form, enabled| {
        (!enabled).then(|| refuse("named off", Form::One(form.into())).enabling(true))
    });
    assert_eq!(
        settled,
        Ok(("FS-login.requirements must cite REQ.".into(), true))
    );
    assert_eq!(passes, 2);
}

/// A pass with more than one candidate chooses none of them.
#[test]
fn a_pass_with_more_than_one_candidate_offers_no_form() {
    let (settled, passes) = settled("Each FS must cite at least 1 FS.", |_, _| {
        let pair = [
            "Each FS must cite a FS.".into(),
            "Each FS must cite FS.".into(),
        ];
        Some(refuse("ambiguous", Form::Two(pair, "or")))
    });
    assert_eq!(settled, Err(false));
    assert_eq!(passes, 1);
}

/// A pass with nothing to put in ends with no form, and says whether what it
/// could not supply is a kind (§FS-rules.3.5.4.4).
#[test]
fn a_pass_with_nothing_to_supply_says_whether_a_kind_is_missing() {
    for kind in [true, false] {
        let (settled, passes) = settled("Each FS must cite at least one POLICY.", |_, _| {
            Some(refuse("missing", Form::None { kind }))
        });
        assert_eq!(settled, Err(kind));
        assert_eq!(passes, 1);
    }
}
