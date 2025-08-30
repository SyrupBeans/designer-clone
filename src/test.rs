use super::*;

#[test]
fn test_traced_clone() {
    let original = 5.traced();

    let clone = original.clone();

    assert_eq!(*original, *clone);
}

#[test]
fn test_traced_clone_with_closure() {
    let result = std::panic::catch_unwind(|| {
        let original = Tr::with_closure(
            5,
            |value| {
                assert_eq!(*value, 5);
                *value = 42;
                false
            },
            |value| {
                assert_eq!(*value, 42);
            },
        );

        let clone = original.clone(); // will panic

        assert_eq!(&*clone as *const _, std::ptr::null());
    });

    let err: Box<&str> = result.unwrap_err().downcast().unwrap();
    println!("{}", err);
    assert_eq!(*err, "Unknown clone not forgiven.");
}

#[test]
fn test_suspend() {
    let original = 42.traced();

    Tr::suspend(&original);
    assert!(original.suspended.get());

    let clone = original.clone();
    assert_eq!(*clone, *original);
}

#[test]
fn test_resume() {
    let result = std::panic::catch_unwind(|| {
        let original = 42.traced();

        Tr::suspend(&original);
        Tr::resume(&original);
        assert!(!original.suspended.get());

        let clone = original.clone(); // will panic
        assert_eq!(*clone, *original);
    });

    let any = result.unwrap_err();
    let err: Box<&str> = any.downcast().unwrap();
    println!("{}", err);
    assert_eq!(*err, "Unknown clone not forgiven.");
}

#[test]
fn test_tag() {
    let tagged = 42.tagged("the answer of life, universe, and everything");
    assert_eq!(*tagged, 42);
    assert_eq!(
        Tag::tag(&tagged),
        &"the answer of life, universe, and everything"
    );
}
