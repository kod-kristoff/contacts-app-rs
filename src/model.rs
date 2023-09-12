use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::sync::RwLock;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Contact {
    id: Option<u64>,
    first: Option<String>,
    last: Option<String>,
    phone: Option<String>,
    pub email: Option<String>,
    #[serde(default)]
    pub errors: HashMap<String, String>,
}

impl Default for Contact {
    fn default() -> Self {
        Self {
            id: None,
            first: None,
            last: None,
            phone: None,
            email: None,
            errors: HashMap::default(),
        }
    }
}

impl Contact {
    pub fn new(
        first: Option<String>,
        last: Option<String>,
        phone: Option<String>,
        email: Option<String>,
    ) -> Self {
        Self {
            first,
            last,
            phone,
            email,
            ..Default::default()
        }
    }

    pub fn validate(&mut self) -> bool {
        if self.email.is_none() {
            self.errors.insert("email".into(), "Email Required".into());
        }
        if self.email.as_ref().is_some_and(|s| s.is_empty()) {
            self.errors.insert("email".into(), "Email Required".into());
        }
        self.errors.len() == 0
    }

    pub fn update(
        &mut self,
        first: Option<String>,
        last: Option<String>,
        phone: Option<String>,
        email: Option<String>,
    ) {
        self.first = first;
        self.last = last;
        self.phone = phone;
        self.email = email;
    }
}
#[async_trait::async_trait]
pub trait ContactRepo {
    async fn all(&self, page: usize) -> Vec<Contact>;
    async fn search(&self, query: &str) -> Vec<Contact>;
    async fn save(&self, contact: Contact) -> Result<(), Contact>;
    async fn find(&self, id: u64) -> Option<Contact>;
    async fn delete(&self, contact: Contact);
}

pub type SharedContactRepo = Arc<dyn ContactRepo + Sync + Send>;

#[derive(Debug, Clone)]
pub struct MemContactRepo {
    path: Option<PathBuf>,
    store: Arc<RwLock<ContactStore>>,
}

#[derive(Debug, Clone)]
pub struct ContactStore {
    contacts: HashMap<u64, Contact>,
}

impl ContactStore {
    pub fn new() -> Self {
        Self {
            contacts: HashMap::new(),
        }
    }

    pub fn from_path(path: &str) -> Self {
        let file = fs::File::open(path).expect("a valid path");
        let reader = io::BufReader::new(file);
        let contact_list: Vec<Contact> = serde_json::from_reader(reader).expect("valid JSON");
        let mut contacts = HashMap::new();
        for contact in contact_list {
            contacts.insert(contact.id.unwrap(), contact);
        }
        Self { contacts }
    }
}

const PAGE_SIZE: usize = 10;

impl MemContactRepo {
    pub fn new() -> Self {
        Self {
            path: None,
            store: Arc::new(RwLock::new(ContactStore::new())),
        }
    }

    pub fn from_path(path: &str) -> Self {
        let store = ContactStore::from_path(path); //.expect("valid JSON");
        Self {
            path: Some(path.into()),
            store: Arc::new(RwLock::new(store)),
        }
    }

    pub fn new_shared() -> SharedContactRepo {
        Arc::new(Self::new())
    }

    pub fn shared_from_path(path: &str) -> SharedContactRepo {
        Arc::new(Self::from_path(path))
    }
}

impl MemContactRepo {
    async fn validate(&self, contact: &mut Contact) -> bool {
        if !contact.validate() {
            return false;
        }
        let contact_email = contact.email.as_ref().unwrap();
        for cont in self.search(contact_email).await {
            if contact.id != cont.id && cont.email.as_ref().unwrap() == contact_email {
                contact
                    .errors
                    .insert("email".into(), "Email Already Exists".into());
                return false;
            }
        }
        true
    }

    async fn max_id(&self) -> u64 {
        self.store
            .read()
            .await
            .contacts
            .keys()
            .max()
            .cloned()
            .unwrap_or(1)
    }

    async fn save_db(&self) {
        let path = self
            .path
            .as_ref()
            .map(|p| p.as_path())
            .unwrap_or_else(|| Path::new("contacts.json"));
        let file = fs::File::create(path).expect("file exist");
        let writer = io::BufWriter::new(file);

        let store = self.store.read().await;
        let contacts: Vec<&Contact> = store.contacts.values().collect();
        serde_json::to_writer(writer, &contacts).expect("writing succeed");
    }
}

#[async_trait::async_trait]
impl ContactRepo for MemContactRepo {
    async fn all(&self, page: usize) -> Vec<Contact> {
        let start = (page - 1) * PAGE_SIZE;
        let end = start + PAGE_SIZE;
        self.store
            .read()
            .await
            .contacts
            .values()
            .skip(start)
            .take(PAGE_SIZE)
            .cloned()
            .collect()
    }

    async fn search(&self, query: &str) -> Vec<Contact> {
        let mut result = Vec::new();
        for contact in self.store.read().await.contacts.values() {
            let match_first = contact
                .first
                .as_ref()
                .map(|s| s.contains(query))
                .unwrap_or(false);
            let match_last = contact
                .last
                .as_ref()
                .map(|s| s.contains(query))
                .unwrap_or(false);
            let match_phone = contact
                .phone
                .as_ref()
                .map(|s| s.contains(query))
                .unwrap_or(false);
            let match_email = contact
                .email
                .as_ref()
                .map(|s| s.contains(query))
                .unwrap_or(false);
            if match_first || match_last || match_phone || match_email {
                result.push(contact.clone());
            }
        }
        result
    }

    async fn save(&self, mut contact: Contact) -> Result<(), Contact> {
        if !self.validate(&mut contact).await {
            return Err(contact);
        }
        if contact.id.is_none() {
            let max_id = self.max_id().await;
            contact.id = Some(max_id + 1);
        }
        self.store
            .write()
            .await
            .contacts
            .insert(contact.id.unwrap(), contact);
        self.save_db().await;
        Ok(())
    }

    async fn find(&self, id: u64) -> Option<Contact> {
        self.store.read().await.contacts.get(&id).cloned()
    }

    async fn delete(&self, contact: Contact) {
        self.store
            .write()
            .await
            .contacts
            .remove(contact.id.as_ref().unwrap());
        self.save_db().await;
    }
}
