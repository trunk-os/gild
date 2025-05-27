mod service {
    use buckle::client::Info;

    use crate::{
        db::models::User,
        server::{Authentication, PingResult},
        testutil::{start_server, TestClient},
    };

    #[tokio::test]
    async fn ping() {
        let mut client = TestClient::new(start_server(None).await.unwrap());
        let results = client.get::<PingResult>("/status/ping").await.unwrap();
        assert!(results.info.is_none());

        let login = User {
            username: "test-login".into(),
            plaintext_password: Some("test-password".into()),
            ..Default::default()
        };
        assert!(client.put::<User, User>("/users", login).await.is_ok());

        client
            .login(Authentication {
                username: "test-login".into(),
                password: "test-password".into(),
            })
            .await
            .unwrap();

        let results = client.get::<PingResult>("/status/ping").await.unwrap();
        assert!(results.info.is_some());
        let info: Info = results.info.unwrap().into();
        assert_ne!(info.uptime, 0);
        assert_ne!(info.available_memory, 0);
        assert_ne!(info.total_memory, 0);
        assert_ne!(info.cpus, 0);
        assert_ne!(info.cpu_usage, 0.0);
        assert!(!info.host_name.is_empty());
        assert!(!info.kernel_version.is_empty());
        assert_ne!(info.load_average, [0.0, 0.0, 0.0]);
        assert_ne!(info.processes, 0);
    }
}

mod user {
    use crate::db::models::User;
    use crate::server::Authentication;
    use crate::testutil::{start_server, TestClient};

    #[tokio::test]
    async fn login_logout() {
        let mut client = TestClient::new(start_server(None).await.unwrap());

        assert!(client.get::<Vec<User>>("/users").await.is_err());

        let login = User {
            username: "test-login".into(),
            plaintext_password: Some("test-password".into()),
            ..Default::default()
        };
        assert!(client.put::<User, User>("/users", login).await.is_ok());

        client
            .login(Authentication {
                username: "test-login".into(),
                password: "test-password".into(),
            })
            .await
            .unwrap();

        assert!(client.get::<Vec<User>>("/users").await.is_ok());
    }

    #[tokio::test]
    async fn first_time_setup() {
        let mut client = TestClient::new(start_server(None).await.unwrap());

        let login = User {
            username: "test-login".into(),
            plaintext_password: Some("test-password".into()),
            ..Default::default()
        };
        assert!(client.put::<User, User>("/users", login).await.is_ok());

        let login = User {
            username: "test-login2".into(),
            plaintext_password: Some("test-password".into()),
            ..Default::default()
        };
        assert!(client.put::<User, User>("/users", login).await.is_err());

        client
            .login(Authentication {
                username: "test-login".into(),
                password: "test-password".into(),
            })
            .await
            .unwrap();

        let login = User {
            username: "test-login2".into(),
            plaintext_password: Some("test-password".into()),
            ..Default::default()
        };
        assert!(client.put::<User, User>("/users", login).await.is_ok());
    }

    #[tokio::test]
    async fn users_validate() {
        let mut client = TestClient::new(start_server(None).await.unwrap());

        let login = User {
            username: "test-login".into(),
            plaintext_password: Some("test-password".into()),
            ..Default::default()
        };
        assert!(client.put::<User, User>("/users", login).await.is_ok());

        client
            .login(Authentication {
                username: "test-login".into(),
                password: "test-password".into(),
            })
            .await
            .unwrap();

        let list = client.get::<Vec<User>>("/users").await.unwrap();
        assert_eq!(list.len(), 1);

        let table: &[User] = &[
            User {
                username: "".into(),
                realname: Some("Erik Hollensbe".into()),
                email: Some("erikhollensbe@proton.me".into()),
                phone: Some("800-867-5309".into()),
                plaintext_password: Some("horlclax".into()),
                ..Default::default()
            },
            User {
                username: "erikh".into(),
                realname: Some("".into()),
                email: Some("erikhollensbe@proton.me".into()),
                phone: Some("800-867-5309".into()),
                plaintext_password: Some("horlclax".into()),
                ..Default::default()
            },
            User {
                username: "erikh".into(),
                realname: Some("Erik Hollensbe".into()),
                email: Some("".into()),
                phone: Some("800-867-5309".into()),
                plaintext_password: Some("horlclax".into()),
                ..Default::default()
            },
            User {
                username: "erikh".into(),
                realname: Some("Erik Hollensbe".into()),
                email: Some("erikhollensbe@proton.me".into()),
                phone: Some("".into()),
                plaintext_password: Some("horlclax".into()),
                ..Default::default()
            },
            User {
                username: "erikh".into(),
                realname: Some("Erik Hollensbe".into()),
                email: Some("erikhollensbe@proton.me".into()),
                phone: Some("800-867-5309".into()),
                plaintext_password: Some("".into()),
                ..Default::default()
            },
            User {
                username: "erikhaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                realname: Some("Erik Hollensbe".into()),
                email: Some("erikhollensbe@proton.me".into()),
                phone: Some("800-867-5309".into()),
                plaintext_password: Some("horlclax".into()),
                ..Default::default()
            },
            User {
                username: "erikh".into(),
                realname: Some("Erik Hollensbeaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into()),
                email: Some("erikhollensbe@proton.me".into()),
                phone: Some("800-867-5309".into()),
                plaintext_password: Some("horlclax".into()),
                ..Default::default()
            },
            User {
                username: "erikh".into(),
                realname: Some("Erik Hollensbe".into()),
                email: Some("erikhollensbe@proton.meaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into()),
                phone: Some("800-867-5309".into()),
                plaintext_password: Some("horlclax".into()),
                ..Default::default()
            },
            User {
                username: "erikh".into(),
                realname: Some("Erik Hollensbe".into()),
                email: Some("erikhollensbe@proton.me".into()),
                phone: Some("800-867-5309aaaaaaaaaaaaaaaaaaaa".into()),
                plaintext_password: Some("horlclax".into()),
                ..Default::default()
            },
            User {
                username: "erikh".into(),
                realname: Some("Erik Hollensbe".into()),
                email: Some("erikhollensbe@proton.me".into()),
                phone: Some("800-867-5309".into()),
                plaintext_password: Some("horlclaxaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into()),
                ..Default::default()
            },
            User {
                username: "er".into(),
                realname: Some("Erik Hollensbe".into()),
                email: Some("erikhollensbe@proton.me".into()),
                phone: Some("800-867-5309".into()),
                plaintext_password: Some("horlclax".into()),
                ..Default::default()
            },
            User {
                username: "erikh".into(),
                realname: Some("Er".into()),
                email: Some("erikhollensbe@proton.me".into()),
                phone: Some("800-867-5309".into()),
                plaintext_password: Some("horlclax".into()),
                ..Default::default()
            },
            User {
                username: "erikh".into(),
                realname: Some("Erik Hollensbe".into()),
                email: Some("e@e".into()),
                phone: Some("800-867-5309".into()),
                plaintext_password: Some("horlclax".into()),
                ..Default::default()
            },
            User {
                username: "erikh".into(),
                realname: Some("Erik Hollensbe".into()),
                email: Some("erikhollensbe@proton.me".into()),
                phone: Some("867-5309".into()),
                plaintext_password: Some("horlclax".into()),
                ..Default::default()
            },
            User {
                username: "erikh".into(),
                realname: Some("Erik Hollensbe".into()),
                email: Some("erikhollensbe@proton.me".into()),
                phone: Some("800-867-5309".into()),
                plaintext_password: Some("horlcla".into()),
                ..Default::default()
            },
            User {
                username: "erikh".into(),
                realname: Some("Erik Hollensbe".into()),
                email: Some("erikhollensbe".into()),
                phone: Some("800-867-5309".into()),
                plaintext_password: Some("horlclax".into()),
                ..Default::default()
            },
        ];

        for (x, item) in table.iter().enumerate() {
            assert!(
                client
                    .put::<User, User>("/users", item.clone())
                    .await
                    .is_err(),
                "#{} succeeded",
                x
            )
        }
    }

    #[tokio::test]
    async fn users_crud() {
        let mut client = TestClient::new(start_server(None).await.unwrap());

        let login = User {
            username: "test-login".into(),
            plaintext_password: Some("test-password".into()),
            ..Default::default()
        };
        assert!(client.put::<User, User>("/users", login).await.is_ok());

        client
            .login(Authentication {
                username: "test-login".into(),
                password: "test-password".into(),
            })
            .await
            .unwrap();

        let list = client.get::<Vec<User>>("/users").await.unwrap();
        assert_eq!(list.len(), 1);

        let table: &[User] = &[
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
                plaintext_password: Some("foobar123".into()),
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
                plaintext_password: Some("mmph1234".into()),
                ..Default::default()
            },
            User {
                username: "dpnvektor".into(),
                realname: Some("Julian Sutter".into()),
                plaintext_password: Some("meh12345".into()),
                ..Default::default()
            },
        ];

        let mut created = Vec::new();

        for item in table.into_iter() {
            let user = client
                .put::<User, User>("/users", item.clone())
                .await
                .unwrap();
            created.push(user);
        }

        for item in table.into_iter() {
            assert!(client
                .put::<User, User>("/users", item.clone())
                .await
                .is_err());
        }

        let list = client.get::<Vec<User>>("/users").await.unwrap();
        assert_eq!(list.len(), table.len() + 1); // add the logged in user

        for item in created.iter() {
            assert_eq!(
                client
                    .get::<User>(&format!("/user/{}", item.id))
                    .await
                    .unwrap(),
                item.clone(),
            );
        }

        // update and fetch and compare
        for mut item in created.clone().into_iter() {
            item.realname = Some("new realname".into());
            client
                .post::<User, ()>(&format!("/user/{}", item.id), item.clone())
                .await
                .unwrap();
            assert_eq!(
                client
                    .get::<User>(&format!("/user/{}", item.id))
                    .await
                    .unwrap(),
                item.clone(),
            );
        }

        for item in created.into_iter() {
            client
                .delete::<()>(&format!("/user/{}", item.id))
                .await
                .unwrap();
        }

        let list = client.get::<Vec<User>>("/users").await.unwrap();
        assert_eq!(list.len(), 1);
    }
}

#[cfg(feature = "zfs")]
mod zfs {
    use std::collections::HashMap;

    use crate::{
        db::models::User,
        server::Authentication,
        testutil::{start_server, TestClient},
    };
    use buckle::client::ZFSStat;

    #[tokio::test]
    async fn zfs_errors() {
        let _ = buckle::testutil::destroy_zpool("errors", None);
        let zpool = buckle::testutil::create_zpool("errors").unwrap();
        let mut client =
            TestClient::new(start_server(Some("buckle-test-gild".into())).await.unwrap());

        let login = User {
            username: "test-login".into(),
            plaintext_password: Some("test-password".into()),
            ..Default::default()
        };
        assert!(client.put::<User, User>("/users", login).await.is_ok());

        client
            .login(Authentication {
                username: "test-login".into(),
                password: "test-password".into(),
            })
            .await
            .unwrap();

        let res = client
            .post::<_, ()>(
                "/zfs/modify_volume",
                buckle::client::ModifyVolume {
                    name: "volume".into(),
                    modifications: buckle::client::Volume {
                        name: "volume2".into(),
                        size: 100000 * 1024 * 1024,
                    },
                },
            )
            .await;

        let err = res.err().unwrap().to_string();

        let map: HashMap<String, String> = serde_json::from_str(&err).unwrap();

        assert_eq!(
            map.get("detail").unwrap().to_string(),
            "Error: cannot open 'buckle-test-gild/volume': dataset does not exist".to_string()
        );

        buckle::testutil::destroy_zpool("errors", Some(&zpool)).unwrap();
    }

    #[tokio::test]
    async fn zfs_basic() {
        let _ = buckle::testutil::destroy_zpool("gild", None);
        let zpool = buckle::testutil::create_zpool("gild").unwrap();
        let mut client =
            TestClient::new(start_server(Some("buckle-test-gild".into())).await.unwrap());

        let login = User {
            username: "test-login".into(),
            plaintext_password: Some("test-password".into()),
            ..Default::default()
        };
        assert!(client.put::<User, User>("/users", login).await.is_ok());

        client
            .login(Authentication {
                username: "test-login".into(),
                password: "test-password".into(),
            })
            .await
            .unwrap();

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

        client
            .post::<_, ()>(
                "/zfs/modify_volume",
                buckle::client::ModifyVolume {
                    name: "volume".into(),
                    modifications: buckle::client::Volume {
                        name: "volume2".into(),
                        size: 5 * 1024 * 1024,
                    },
                },
            )
            .await
            .unwrap();

        let result: Vec<ZFSStat> = client.post("/zfs/list", "volume2").await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "volume2");
        assert_eq!(result[0].full_name, "buckle-test-gild/volume2");
        assert_ne!(result[0].size, 0);
        assert!(result[0].size < 6 * 1024 * 1024 && result[0].size > 4 * 1024 * 1024);
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
            .post::<_, ()>(
                "/zfs/modify_dataset",
                buckle::client::ModifyDataset {
                    name: "dataset".into(),
                    modifications: buckle::client::Dataset {
                        name: "dataset2".into(),
                        quota: Some(5 * 1024 * 1024),
                    },
                },
            )
            .await
            .unwrap();

        let result: Vec<ZFSStat> = client.post("/zfs/list", "dataset2").await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "dataset2");
        assert_eq!(result[0].full_name, "buckle-test-gild/dataset2");
        assert_ne!(result[0].size, 0);
        assert_ne!(result[0].avail, 0);
        assert_ne!(result[0].refer, 0);
        assert_ne!(result[0].used, 0);
        assert_eq!(
            result[0].mountpoint,
            Some("/buckle-test-gild/dataset2".into())
        );

        client
            .post::<_, ()>("/zfs/destroy", "dataset2")
            .await
            .unwrap();
        let result: Vec<ZFSStat> = client.post("/zfs/list", "dataset2").await.unwrap();
        assert_eq!(result.len(), 0);
        let result: Vec<ZFSStat> = client.post("/zfs/list", "").await.unwrap();
        assert_eq!(result.len(), 1);
        client
            .post::<_, ()>("/zfs/destroy", "volume2")
            .await
            .unwrap();
        let result: Vec<ZFSStat> = client.post("/zfs/list", "volume2").await.unwrap();
        assert_eq!(result.len(), 0);

        buckle::testutil::destroy_zpool("gild", Some(&zpool)).unwrap();
    }
}
