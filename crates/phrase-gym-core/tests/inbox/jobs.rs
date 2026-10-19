use std::sync::Arc;

use phrase_gym_contracts::inbox::{DeliveryStatus, JobFailure};
use phrase_gym_core::inbox::jobs::Jobs;
use uuid::Uuid;

use super::{TestClock, inbox, input, support::Fixture};

#[test]
fn claims_are_scoped_idempotent_fenced_and_bounded() {
    let f = Fixture::new();
    let inbox = inbox(&f);
    let receipt = inbox.submit("sender", &input(&f)).unwrap();
    let clock = Arc::new(TestClock::new());
    let jobs = Jobs::new(f.database.clone(), clock.clone());
    let id = Uuid::new_v4();
    let first = jobs.claim("worker", id).unwrap().unwrap();
    assert_eq!(first, jobs.claim("worker", id).unwrap().unwrap());
    assert!(jobs.claim("other-worker", id).is_err());
    assert_eq!(first.receipt, receipt);
    assert_eq!(first.profile.revision, receipt.profile_revision);
    assert!(!first.guide.is_empty());
    assert!(
        jobs.claim("other-worker", Uuid::new_v4())
            .unwrap()
            .is_none()
    );
    assert!(jobs.renew("other-worker", &first.fence).is_err());
    clock.advance(100);
    jobs.renew("worker", &first.fence).unwrap();
    clock.advance(121);
    let second = jobs.claim("other-worker", Uuid::new_v4()).unwrap().unwrap();
    assert_eq!(second.fence.fence, first.fence.fence + 1);
    assert!(jobs.renew("worker", &first.fence).is_err());
    jobs.fail(
        "other-worker",
        &JobFailure {
            fence: second.fence.clone(),
            reason: "Unknown meaning; supply context".into(),
        },
    )
    .unwrap();
    assert_eq!(
        inbox
            .status("sender", receipt.generation_id)
            .unwrap()
            .status,
        DeliveryStatus::Failed
    );
    inbox.retry("sender", receipt.generation_id).unwrap();
    let third = jobs.claim("worker", Uuid::new_v4()).unwrap().unwrap();
    clock.advance(120);
    assert!(jobs.renew("worker", &third.fence).is_err());
    assert!(jobs.claim("worker", Uuid::new_v4()).unwrap().is_none());
    assert!(inbox.retry("sender", receipt.generation_id).is_err());
    assert_eq!(
        inbox
            .status("sender", receipt.generation_id)
            .unwrap()
            .status,
        DeliveryStatus::Failed
    );
}

#[test]
fn lease_renewal_cannot_extend_attempt_deadline_and_cancel_or_replacement_fences_workers() {
    let f = Fixture::new();
    let inbox = inbox(&f);
    let mut input = input(&f);
    let receipt = inbox.submit("sender", &input).unwrap();
    let clock = Arc::new(TestClock::new());
    let jobs = Jobs::new(f.database.clone(), clock.clone());
    let first = jobs.claim("worker", Uuid::new_v4()).unwrap().unwrap();
    for _ in 0..5 {
        clock.advance(100);
        jobs.renew("worker", &first.fence).unwrap();
    }
    clock.advance(100);
    assert!(jobs.renew("worker", &first.fence).is_err());
    let second = jobs.claim("worker", Uuid::new_v4()).unwrap().unwrap();
    input.submission_id = Uuid::new_v4();
    input.expected_generation = Some(1);
    inbox.submit("sender", &input).unwrap();
    assert!(jobs.renew("worker", &second.fence).is_err());
    assert_eq!(
        inbox
            .status("sender", receipt.generation_id)
            .unwrap()
            .status,
        DeliveryStatus::Superseded
    );
    let third = jobs.claim("worker", Uuid::new_v4()).unwrap().unwrap();
    inbox.cancel("sender", third.receipt.generation_id).unwrap();
    assert!(jobs.renew("worker", &third.fence).is_err());
}

#[test]
fn concurrent_claims_have_one_winner_and_daily_budget_survives_new_service() {
    let f = Fixture::new();
    let inbox = inbox(&f);
    let clock = Arc::new(TestClock::new());
    inbox.submit("sender", &input(&f)).unwrap();
    std::thread::scope(|scope| {
        let a = scope.spawn(|| {
            Jobs::new(f.database.clone(), clock.clone())
                .claim("a", Uuid::new_v4())
                .unwrap()
        });
        let b = scope.spawn(|| {
            Jobs::new(f.database.clone(), clock.clone())
                .claim("b", Uuid::new_v4())
                .unwrap()
        });
        assert_eq!(
            usize::from(a.join().unwrap().is_some()) + usize::from(b.join().unwrap().is_some()),
            1
        );
    });
    for n in 1..=20 {
        let mut submission = input(&f);
        submission.item_id = format!("lesson-{n}");
        inbox.submit("sender", &submission).unwrap();
        let result = Jobs::new(f.database.clone(), clock.clone()).claim("worker", Uuid::new_v4());
        if n < 20 {
            assert!(result.unwrap().is_some());
        } else {
            assert!(result.is_err());
        }
    }
    clock.advance(86_400);
    assert!(
        Jobs::new(f.database.clone(), clock)
            .claim("worker", Uuid::new_v4())
            .unwrap()
            .is_some()
    );
}
