mod service {
    use crate::testutil::{start_server, TestClient};

    #[tokio::test]
    async fn ping() {
        let client = TestClient::new(start_server(None).await.unwrap());
        assert!(client.get::<()>("/status/ping").await.is_ok());
    }
}

mod user {
    use crate::db::models::User;
    use crate::testutil::{start_server, TestClient};

    #[tokio::test]
    async fn users_crud() {
        let client = TestClient::new(start_server(None).await.unwrap());
        let list = client.get::<Vec<User>>("/users").await.unwrap();
        assert_eq!(list.len(), 0);

        let table: &mut [User] = &mut [
            User {
                username: "erikh".into(),
                realname: Some("Erik Hollensbe".into()),
                email: Some("erikhollensbe@proton.me".into()),
                phone: Some("800-867-5309".into()),
                plaintext_password: Some("horlclax".into()),
                ..Default::default()
            },
            User {
                username: "scarlett".into(),
                realname: Some("Scarlett Hollensbe".into()),
                email: Some("scarlett@hollensbe.org".into()),
                phone: None,
                plaintext_password: Some("foobar".into()),
                ..Default::default()
            },
            User {
                username: "cmaujean".into(),
                realname: Some("Christopher Maujean".into()),
                email: Some("christopher@maujean.org".into()),
                plaintext_password: Some("pooprocket".into()),
                ..Default::default()
            },
            User {
                username: "day".into(),
                realname: Some("Day Waterbury".into()),
                plaintext_password: Some("mmph".into()),
                ..Default::default()
            },
            User {
                username: "dpnvektor".into(),
                realname: Some("Julian Sutter".into()),
                plaintext_password: Some("meh".into()),
                ..Default::default()
            },
        ];

        let mut created = Vec::new();

        for item in table.into_iter() {
            item.set_password(item.plaintext_password.clone().unwrap())
                .unwrap();
            let user = client
                .put::<User, User>("/users", item.clone())
                .await
                .unwrap();
            created.push(user);
        }

        let list = client.get::<Vec<User>>("/users").await.unwrap();
        assert_eq!(list.len(), table.len());

        for item in created.into_iter() {
            assert_eq!(
                client
                    .get::<User>(&format!("/user/{}", item.id))
                    .await
                    .unwrap(),
                item.clone(),
            );
        }
    }
}

#[cfg(feature = "zfs")]
mod zfs {
    use crate::testutil::{start_server, TestClient};
    use buckle::client::ZFSStat;

    #[tokio::test]
    async fn zfs_basic() {
        let _ = buckle::testutil::destroy_zpool("gild", None);
        let zpool = buckle::testutil::create_zpool("gild").unwrap();
        let client = TestClient::new(start_server(Some("buckle-test-gild".into())).await.unwrap());
        let result: Vec<ZFSStat> = client.post("/zfs/list", "").await.unwrap();
        assert_eq!(result.len(), 0);
        client
            .post::<_, ()>(
                "/zfs/create_dataset",
                buckle::client::Dataset {
                    name: "dataset".into(),
                    quota: None,
                },
            )
            .await
            .unwrap();
        let result: Vec<ZFSStat> = client.post("/zfs/list", "").await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "dataset");
        assert_eq!(result[0].full_name, "buckle-test-gild/dataset");
        assert_ne!(result[0].size, 0);
        assert_ne!(result[0].avail, 0);
        assert_ne!(result[0].refer, 0);
        assert_ne!(result[0].used, 0);
        assert_eq!(
            result[0].mountpoint,
            Some("/buckle-test-gild/dataset".into())
        );
        client
            .post::<_, ()>(
                "/zfs/create_volume",
                buckle::client::Volume {
                    name: "volume".into(),
                    size: 100 * 1024 * 1024,
                },
            )
            .await
            .unwrap();
        let result: Vec<ZFSStat> = client.post("/zfs/list", "").await.unwrap();
        assert_eq!(result.len(), 2);
        let result: Vec<ZFSStat> = client.post("/zfs/list", "volume").await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "volume");
        assert_eq!(result[0].full_name, "buckle-test-gild/volume");
        assert_ne!(result[0].size, 0);
        assert_ne!(result[0].avail, 0);
        assert_ne!(result[0].refer, 0);
        assert_ne!(result[0].used, 0);
        assert_eq!(result[0].mountpoint, None);

        let result: Vec<ZFSStat> = client.post("/zfs/list", "dataset").await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "dataset");
        assert_eq!(result[0].full_name, "buckle-test-gild/dataset");
        assert_ne!(result[0].size, 0);
        assert_ne!(result[0].avail, 0);
        assert_ne!(result[0].refer, 0);
        assert_ne!(result[0].used, 0);
        assert_eq!(
            result[0].mountpoint,
            Some("/buckle-test-gild/dataset".into())
        );

        client
            .post::<_, ()>("/zfs/destroy", "dataset")
            .await
            .unwrap();
        let result: Vec<ZFSStat> = client.post("/zfs/list", "dataset").await.unwrap();
        assert_eq!(result.len(), 0);
        let result: Vec<ZFSStat> = client.post("/zfs/list", "").await.unwrap();
        assert_eq!(result.len(), 1);
        client
            .post::<_, ()>("/zfs/destroy", "volume")
            .await
            .unwrap();
        let result: Vec<ZFSStat> = client.post("/zfs/list", "volume").await.unwrap();
        assert_eq!(result.len(), 0);

        buckle::testutil::destroy_zpool("gild", Some(&zpool)).unwrap();
    }
}
