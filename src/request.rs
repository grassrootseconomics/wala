use std::path::Path;
use std::str::FromStr;
use tiny_http::{
    Method,
    Response,
    Request,
    StatusCode,
};
use crate::record::{
    put_immutable,
    put_mutable,
    get as get_record,
    ResourceKey,
    RequestResult,
    RequestResultType,
};
use crate::auth::{
    AuthResult,
};
use std::io::Read;

use log::{
    debug,
    error,
};

pub fn process_method(method: &Method, url: String, mut f: impl Read, expected_size: usize, path: &Path, auth_result: AuthResult) -> RequestResult {
    match method {
        Method::Put => {
            if !auth_result.valid() {
                return RequestResult{
                    typ: RequestResultType::AuthError,
                    v: None,
                    f: None,
                };
            }
            if auth_result.active() {
                let res: RequestResult;
                let rk = ResourceKey::from_str(url.as_str()).unwrap();
                debug!("mutable put, authenticated as {:?} using mutable key {} -> {}", auth_result, &url, &rk);
                let ptr = rk.pointer_for(&auth_result);
                match put_mutable(ptr, path, f, expected_size) {
                    Ok(v) => {
                        let digest_hex = hex::encode(v.digest);
                        res = RequestResult{
                            typ: RequestResultType::Changed,
                            v: Some(digest_hex),
                            f: None,
                        };
                    },
                    Err(e) => {
                        let err_str = format!("{:?}", e);
                        error!("{}", err_str);
                        res = RequestResult {
                            typ: RequestResultType::RecordError,
                            v: Some(String::from(err_str)),
                            f: None,
                        };
                    },
                };
                return res;
            } else {
                debug!("immutable put");
                let res: RequestResult;
                match put_immutable(path, f, expected_size) {
                    Ok(v) => {
                        let digest_hex = hex::encode(v.digest);
                        res = RequestResult{
                            typ: RequestResultType::Changed,
                            v: Some(digest_hex),
                            f: None,
                        };
                    },
                    Err(e) => {
                        let err_str = format!("{}", e);
                        res = RequestResult {
                            typ: RequestResultType::RecordError,
                            v: Some(String::from(err_str)),
                            f: None,
                        };
                    },
                };
                return res;
            }
        },
        Method::Get => {
            let digest = match hex::decode(&url) {
                Err(e) => {
                    let err_str = format!("{}", e);
                    return RequestResult {
                        typ: RequestResultType::InputError,
                        v: Some(String::from(err_str)),
                        f: None,
                    };
                },
                Ok(v) => {
                    v
                },
            };

            let full_path_buf = path.join(&url);
            debug!("url {} resolved to {:?}", &url, &full_path_buf);

            match get_record(digest, full_path_buf.as_path()) {
                Some(v) => {
                    return RequestResult {
                        typ: RequestResultType::Found,
                        v: None, //Some(String::new()),
                        f: Some(v),
                    };
                },
                None => {
                    debug!("nooonn");
                    return RequestResult {
                        typ: RequestResultType::RecordError,
                        v: Some(String::new()),
                        f: None,
                    };
                },
            };
        },
        _ => {},
    };
    RequestResult {
        typ: RequestResultType::InputError,
        v: Some(String::new()),
        f: None,
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use tiny_http::Method;
    use super::process_method;
    use std::fs::{
        read,
        write,
        File,
    };
    use std::path::Path;
    use crate::auth::AuthResult;
    use crate::record::RequestResultType;
    use env_logger;


    #[test]
    fn test_get_ok() {
        let d = tempdir().unwrap();
        let url = String::from("deadbeef");
        let data = "foobar";

        let fp = d.path().join(&url); //.as_path().to_string().unwrap();
        write(&fp, data);
        let f = File::open(&fp).unwrap();

        let method = Method::Get;

        let auth = AuthResult {
            identity: vec!(),
            error: false,
        };

        let res = process_method(&method, url, f, 6, &d.path(), auth);
        assert_eq!(res.typ, RequestResultType::Found);
    }

    #[test]
    fn test_get_bogus() {
        let d = tempdir().unwrap();
        let url = String::from("teadbeef");
        let data = "foobar";

        let fp = d.path().join(&url); //.as_path().to_string().unwrap();
        write(&fp, data);
        let f = File::open(&fp).unwrap();

        let method = Method::Get;

        let auth = AuthResult {
            identity: vec!(),
            error: false,
        };

        let res = process_method(&method, url, f, 6, &d.path(), auth);
        assert_eq!(res.typ, RequestResultType::InputError);
    }

    #[test]
    fn test_put_immutable() {
        let d = tempdir().unwrap();
        let mut url = String::from("deadbeef");
        let data = "foobar";

        let fp = d.path().join(&url); //.as_path().to_string().unwrap();
        write(&fp, data);
        let f = File::open(&fp).unwrap();

        let method = Method::Put;

        let auth = AuthResult {
            identity: vec!(),
            error: false,
        };
    
        url = String::new();
        let res = process_method(&method, url, f, 6, &d.path(), auth);
        assert_eq!(res.typ, RequestResultType::Changed);

        let content_ref = String::from("c3ab8ff13720e8ad9047dd39466b3c8974e592c2fa383d4a3960714caef0c4f2");
        assert_eq!(res.v.unwrap(), content_ref);

    }

    #[test]
    fn test_put_mutable() {
        let d = tempdir().unwrap();
        let url = String::from("deadbeef");
        let data = "foobar";

        let fp = d.path().join(&url); //.as_path().to_string().unwrap();
        write(&fp, data);
        let f = File::open(&fp).unwrap();

        let method = Method::Put;

        let auth = AuthResult {
            identity: vec!(0x66, 0x6f, 0x6f),
            error: false,
        };

        let res = process_method(&method, url, f, 6, &d.path(), auth);
        assert_eq!(res.typ, RequestResultType::Changed);

        let content_ref = String::from("129208a8eac1bedd060645411baaae4aabc5d9e4c858942defe139b5ba15aba6");
        assert_eq!(res.v.unwrap(), content_ref);

        let fp_immutable = d.path().join(&content_ref);
        let r = read(fp_immutable).unwrap();

        assert_eq!(data.as_bytes(), r);
    }

    #[test]
    fn test_put_mutable_noauth() {
        let d = tempdir().unwrap();
        let url = String::from("deadbeef");
        let data = "foobar";

        let fp = d.path().join(&url); //.as_path().to_string().unwrap();
        write(&fp, data);
        let f = File::open(&fp).unwrap();

        let method = Method::Put;

        let auth = AuthResult {
            identity: vec!(0x2a),
            error: true,
        };

        let res = process_method(&method, url, f, 6, &d.path(), auth);
        assert_eq!(res.typ, RequestResultType::AuthError);
    }
}
