use lru::LruCache;
use std::sync::{Arc, Mutex};
use libra::libra_state_view::StateView;
use libra::libra_types::access_path::AccessPath;
use anyhow::Error;
use libra::move_vm_runtime::data_cache::RemoteCache;
use libra::libra_vm::errors::VMResult;
use crate::{DataSource, Clear};

const CODE_TAG: u8 = 0;

#[derive(Debug, Clone)]
pub struct ModuleCache<D>
where
    D: DataSource,
{
    inner: D,
    cache: Arc<Mutex<LruCache<AccessPath, Vec<u8>>>>,
}

impl<D> ModuleCache<D>
where
    D: DataSource,
{
    pub fn new(inner: D, cache_size: usize) -> ModuleCache<D> {
        ModuleCache {
            inner,
            cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
        }
    }
}

impl<D> StateView for ModuleCache<D>
where
    D: DataSource,
{
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>, Error> {
        if access_path.path[0] == CODE_TAG {
            let module = {
                let mut cache = self.cache.lock().unwrap();
                cache.get(access_path).map(|m| m.to_vec())
            };

            if let Some(module) = module {
                Ok(Some(module))
            } else {
                let module = StateView::get(&self.inner, access_path)?;
                if let Some(module) = module {
                    let mut cache = self.cache.lock().unwrap();
                    cache.put(access_path.clone(), module.to_vec());
                    Ok(Some(module))
                } else {
                    Ok(None)
                }
            }
        } else {
            StateView::get(&self.inner, access_path)
        }
    }

    fn multi_get(&self, access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>, Error> {
        access_paths
            .iter()
            .map(|path| StateView::get(self, path))
            .collect()
    }

    fn is_genesis(&self) -> bool {
        self.inner.is_genesis()
    }
}

impl<D> Clear for ModuleCache<D>
where
    D: DataSource,
{
    fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        self.inner.clear();
    }
}

impl<D> RemoteCache for ModuleCache<D>
where
    D: DataSource,
{
    fn get(&self, access_path: &AccessPath) -> VMResult<Option<Vec<u8>>> {
        RemoteCache::get(&self.inner, access_path)
    }
}

impl<D> DataSource for ModuleCache<D> where D: DataSource {}
