use welds::state::DbState;

use super::User;
use crate::testutil::*;

#[tokio::test]
async fn user_password() {
    let db = make_config(None, None)
        .await
        .unwrap()
        .get_db()
        .await
        .unwrap();

    let mut user = User::new();
    user.username = "erikh".into();
    assert!(user.set_password("horlclax".into()).is_ok());
    assert_ne!(
        String::from_utf8(user.password.clone()).unwrap(),
        "horlclax".to_string()
    );
    assert!(user.save(&db.handle).await.is_ok());

    let user2 = User::find_by_id(&db.handle, user.id).await.unwrap();
    assert!(user2.is_some());
    let user2 = user2.unwrap();
    assert_eq!(user2.username, user.username);
    assert_eq!(user2.password, user.password);
    assert_eq!(user2.id, user.id);

    assert!(user.login("test".into()).is_err());
    assert!(user.login("horlclax".into()).is_ok());
}

#[tokio::test]
async fn user_basic() {
    let db = make_config(None, None)
        .await
        .unwrap()
        .get_db()
        .await
        .unwrap();

    let table: &mut [DbState<User>] = &mut [
        DbState::new_uncreated(User {
            id: 0,
            username: "erikh".into(),
            realname: Some("Erik Hollensbe".into()),
            email: Some("erikhollensbe@proton.me".into()),
            phone: Some("800-867-5309".into()),
            password: Vec::new(),
            plaintext_password: Some("horlclax".into()),
        }),
        DbState::new_uncreated(User {
            id: 0,
            username: "scarlett".into(),
            realname: Some("Scarlett Hollensbe".into()),
            email: Some("scarlett@hollensbe.org".into()),
            phone: None,
            password: Vec::new(),
            plaintext_password: Some("foobar".into()),
        }),
        DbState::new_uncreated(User {
            id: 0,
            username: "cmaujean".into(),
            realname: Some("Christopher Maujean".into()),
            email: Some("christopher@maujean.org".into()),
            phone: None,
            password: Vec::new(),
            plaintext_password: Some("pooprocket".into()),
        }),
        DbState::new_uncreated(User {
            id: 0,
            username: "day".into(),
            realname: Some("Day Waterbury".into()),
            email: None,
            phone: None,
            password: Vec::new(),
            plaintext_password: Some("mmph".into()),
        }),
        DbState::new_uncreated(User {
            id: 0,
            username: "dpnvektor".into(),
            realname: Some("Julian Sutter".into()),
            email: None,
            phone: None,
            password: Vec::new(),
            plaintext_password: Some("meh".into()),
        }),
    ];

    for item in table.into_iter() {
        let pw = item.plaintext_password.clone().unwrap();
        item.set_password(pw).unwrap();
        assert_ne!(item.password.len(), 0);
        assert_ne!(
            &item.password,
            item.plaintext_password.clone().unwrap().as_bytes(),
        );
        assert!(item.save(&db.handle).await.is_ok());
        assert_ne!(item.id, 0);
    }

    assert_eq!(User::all().count(&db.handle).await.unwrap(), 5);

    for item in table.into_iter() {
        assert_eq!(
            User::all()
                .where_col(|c| c.username.equal(&item.username))
                .run(&db.handle)
                .await
                .unwrap()
                .first()
                .unwrap()
                .id,
            item.id
        )
    }

    for item in table.into_iter() {
        assert!(item.delete(&db.handle).await.is_ok());
    }

    assert_eq!(User::all().count(&db.handle).await.unwrap(), 0);
}
