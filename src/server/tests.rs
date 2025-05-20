mod service {
    use crate::testutil::{start_server, TestClient};

    #[tokio::test]
    async fn ping() {
        let client = TestClient::new(start_server(None).await.unwrap());
        assert!(client.get::<()>("/status/ping").await.is_ok());
    }
}

mod user {
    #[allow(unused)]
    use crate::testutil::{start_server, TestClient};

    #[tokio::test]
    async fn users_crud() {
        let client = TestClient::new(start_server(None).await.unwrap());
        assert!(client.get::<()>("/status/ping").await.is_ok());
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
