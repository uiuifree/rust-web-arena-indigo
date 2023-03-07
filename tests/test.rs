use web_arena_indigo::{UpdateSshKeyRequest, WebArenaIndigoApi};

#[tokio::test]
async fn instance() {
    dotenv::dotenv().expect("check api key");
    let k1 = std::env::var("C_KEY").unwrap_or("hoge".to_string());
    let k2 = std::env::var("C_SECRET_KEY").unwrap_or("hoge".to_string());
    let mut api = WebArenaIndigoApi::new(k1.as_str(), k2.as_str());
    let res = api.update_access_token().await;
    assert!(res.is_ok());
    // let res = api.instance().instance_type_list().await;
    // assert!(res.is_ok());
    // let test_instance_type_id = 1;
    // let test_ssh_key_id = 15532;
    // let test_region_id = 1;
    // let test_os_id = 13; // Ubuntu 22.04
    // let test_plan = 1;
    // let res = api.instance().region_list(test_instance_type_id).await;
    // assert!(res.is_ok());
    // let res = api.instance().instance_specification(test_instance_type_id).await;
    // println!("{:?}", res);
    // assert!(res.is_ok());
    // let res = api.instance().os_list(test_instance_type_id).await;
    // assert!(res.is_ok(),"{:?}",res);
    // let res = api.instance().create_instance(RequestCreateInstance{
    //     ssh_key_id: test_ssh_key_id,
    //     region_id: test_region_id,
    //     os_id: test_os_id,
    //     instance_plan: test_plan,
    //     instance_name: "test_instance_name3".to_string(),
    // }).await;
    // println!("{:?}", res);
    // assert!(res.is_ok(),"{:?}",res);
    // 476020
    let res = api.instance().get_instance_list().await;
    assert!(res.is_ok(), "{:?}", res);
    for row in res.unwrap() {
        println!("{} {} {:?}", row.instance_name, row.status, row.ipaddress);
    }
    // let res = api.instance().update_instance_status(476020,InstanceStatus::Destroy).await;
    // println!("{:?}", res);
}

#[tokio::test]
async fn test_ssh() {
    dotenv::dotenv().expect("check api key");
    let k1 = std::env::var("C_KEY").unwrap_or("hoge".to_string());
    let k2 = std::env::var("C_SECRET_KEY").unwrap_or("hoge".to_string());
    let mut api = WebArenaIndigoApi::new(k1.as_str(), k2.as_str());
    let res = api.update_access_token().await;
    assert!(res.is_ok(), "{}", res.err().unwrap().to_string());

    let res = api.ssh().ssh_key_list().await;
    assert!(res.is_ok(), "{}", res.err().unwrap().to_string());
    let res = api.ssh().active_ssh_key_list().await;
    assert!(res.is_ok(), "{}", res.err().unwrap().to_string());
    let res = api.ssh().create_ssh_key("test", "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQCTOBwEzlxQhE++5aYEGKpl6Ye8hRD9hjHj/owiu4VQXA2Ew4PLI9/1sbR/75sJ3Ku7x5+LD8QFU2xEDxAPeDI9+4v8vKswc1OofhAucy+GzLE8HzLHxv/0xfGHpMePYETLmHLr+pLhOCpFam1D58AatqMBJzYdXoagHYQHQ3yj4Iv1EWlDUe8SOC69/l5bGdevRkapFBlaQPgL7c/x3BSSV/bquy3fKk4w/o5fDDorOUtnCVt/fB7nciGhBez0K9ce1YMFkF0K/dBrhEFMNsJvElG45GVjc99QGcSn3oEAwzqT/4qzEoY+5OuBXIb7a13ozmdK0ilPWkdu28U4DnCCGw7x4iroEmc+KsBbIvSZOBDVfb7biAOJyXuAz6JPVVCmD6xOA/XlJIaecV+wBWvC+LZ/TO4tKCo/A4PKc7nHmclSH3T+acUu8WUuUmnSjQmYIOukFq9PwamqJkReGgNMrHpyBJFSX1hHYVC7mQdo3dhbpYY1+re8UVIccD75Okk= uiuifree@uiuifree-ubuntu").await;
    assert!(res.is_ok(), "{}", res.err().unwrap().to_string());
    let key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQCTOBwEzlxQhE++5aYEGKpl6Ye8hRD9hjHj/owiu4VQXA2Ew4PLI9/1sbR/75sJ3Ku7x5+LD8QFU2xEDxAPeDI9+4v8vKswc1OofhAucy+GzLE8HzLHxv/0xfGHpMePYETLmHLr+pLhOCpFam1D58AatqMBJzYdXoagHYQHQ3yj4Iv1EWlDUe8SOC69/l5bGdevRkapFBlaQPgL7c/x3BSSV/bquy3fKk4w/o5fDDorOUtnCVt/fB7nciGhBez0K9ce1YMFkF0K/dBrhEFMNsJvElG45GVjc99QGcSn3oEAwzqT/4qzEoY+5OuBXIb7a13ozmdK0ilPWkdu28U4DnCCGw7x4iroEmc+KsBbIvSZOBDVfb7biAOJyXuAz6JPVVCmD6xOA/XlJIaecV+wBWvC+LZ/TO4tKCo/A4PKc7nHmclSH3T+acUu8WUuUmnSjQmYIOukFq9PwamqJkReGgNMrHpyBJFSX1hHYVC7mQdo3dhbpYY1+re8UVIccD75Okk= uiuifree@uiuifree-ubuntu";
    let res = api.ssh().create_ssh_key("test", key).await;
    assert!(res.is_ok(), "{}", res.err().unwrap().to_string());
    let ssh_key_id = res.unwrap().ssh_key.id;
    let res = api.ssh().retrieve_ssh_key(ssh_key_id).await;
    assert!(res.is_ok(), "{}", res.err().unwrap().to_string());
    let res = api.ssh().update_ssh_key(ssh_key_id, UpdateSshKeyRequest {
        ssh_name: "test2".to_string(),
        ssh_key: key.to_string(),
        ssh_key_status: Default::default(),
    }).await;
    assert!(res.is_ok(), "{}", res.err().unwrap().to_string());
    let res = api.ssh().destroy_ssh_key(30493).await;
    assert!(res.is_ok(), "{}", res.err().unwrap().to_string());
}
