use bytes::{Bytes, BytesMut, Buf};

#[derive(Debug, PartialEq)]
pub enum Value {
    SimpleString(String),
    BulkString(Bytes),
    Array(Vec<Value>),
}

pub fn parse_resp(buf: &mut BytesMut) -> Option<Value> {
    if buf.is_empty() {
        log::error!("Buffer is empty");
        return None;
    }

    match buf[0] {
        b'+' => parse_simple_string(buf),
        b'$' => parse_bulk_string(buf),
        b'*' => parse_array(buf),
        _ => {
            log::error!("Unknown RESP type: {}", buf[0] as char);
            None
        }
    }
}

fn parse_simple_string(buf: &mut BytesMut) -> Option<Value> {
    if let Some(pos) = buf.as_ref().windows(2).position(|w| w == b"\r\n") {
        if pos < 1 {
            return None;
        }
        let data = buf.split_to(pos + 2);
        let s = String::from_utf8_lossy(&data[1..pos]);
        Some(Value::SimpleString(s.to_string()))
    } else {
        log::error!("Incomplete simple string");
        None
    }
}

fn parse_bulk_string(buf: &mut BytesMut) -> Option<Value> {
    if let Some(crlf_pos) = buf.as_ref().windows(2).position(|w| w == b"\r\n") {
        if crlf_pos < 2 {
            return None;
        }
        let len_str = String::from_utf8_lossy(&buf[1..crlf_pos]);
        let len: usize = match len_str.parse() {
            Ok(l) => l,
            Err(_) => {
                log::error!("Invalid bulk string length");
                return None;
            }
        };
        buf.advance(crlf_pos + 2);
        if buf.len() < len + 2 {
            log::error!("Incomplete bulk string data");
            return None;
        }
        let data = buf.split_to(len).freeze();
        if &buf[..2] != b"\r\n" {
            log::error!("Missing CRLF after bulk string");
            return None;
        }
        buf.advance(2);
        Some(Value::BulkString(data))
    } else {
        log::error!("Incomplete bulk string header");
        None
    }
}

fn parse_array(buf: &mut BytesMut) -> Option<Value> {
    if let Some(crlf_pos) = buf.as_ref().windows(2).position(|w| w == b"\r\n") {
        if crlf_pos < 2 {
            return None;
        }
        let count_str = String::from_utf8_lossy(&buf[1..crlf_pos]);
        let count: usize = match count_str.parse() {
            Ok(c) => c,
            Err(_) => {
                log::error!("Invalid array count");
                return None;
            }
        };
        buf.advance(crlf_pos + 2);
        let mut array = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(val) = parse_resp(buf) {
                array.push(val);
            } else {
                return None;
            }
        }
        Some(Value::Array(array))
    } else {
        log::error!("Incomplete array header");
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_redis_array() {
        let mut buf = BytesMut::from("*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");
        let result = parse_resp(&mut buf);
        assert!(result.is_some());
        if let Some(Value::Array(arr)) = result {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::BulkString(Bytes::from("foo")));
            assert_eq!(arr[1], Value::BulkString(Bytes::from("bar")));
        } else {
            panic!("Expected array");
        }
    }
}