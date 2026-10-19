use std::sync::Arc;

use phrase_gym_core::english::EnglishPackages;
use phrase_gym_core::error::CoreError;
use phrase_gym_core::inbox::{delivery::Delivery, jobs::Jobs};
use phrase_gym_core::media::store::MediaStore;
use uuid::Uuid;

use super::{TestClock, inbox, input, output, support::Fixture};

#[test]
fn publication_identity_retains_its_payload_across_claim_replacement() {
    let f = Fixture::new();
    inbox(&f).submit("sender", &input(&f)).unwrap();
    let clock = Arc::new(TestClock::new());
    let jobs = Jobs::new(f.database.clone(), clock.clone());
    let delivery = Delivery::new(
        f.database.clone(),
        Arc::new(EnglishPackages::new(
            f.database.clone(),
            Arc::new(MediaStore::open(f.directory.path()).unwrap()),
            f.profiles.clone(),
        )),
        clock.clone(),
    );
    let first = jobs.claim("worker", Uuid::new_v4()).unwrap().unwrap();
    let mut output = output(&f, &first);
    delivery.stage("worker", &output).unwrap();
    clock.advance(120);
    let second = jobs.claim("worker", Uuid::new_v4()).unwrap().unwrap();
    output.fence = second.fence.clone();
    let mut conflicting = output.clone();
    conflicting.package.lesson.title = "Another meeting".into();
    assert!(matches!(
        delivery.stage("worker", &conflicting),
        Err(CoreError::Conflict(_))
    ));
    delivery.stage("worker", &output).unwrap();
    let receipt = delivery.publish("worker", &second.fence).unwrap();
    assert_eq!(receipt.publication_id, output.publication_id);
    assert!(delivery.publish("worker", &first.fence).is_err());
}
