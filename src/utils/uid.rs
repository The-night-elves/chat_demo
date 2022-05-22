use uuid::Uuid;

pub fn generate_uid() -> String {
    Uuid::new_v4().to_string()
}


