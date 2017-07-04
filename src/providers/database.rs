use iron::prelude::*;
use iron::{typemap, BeforeMiddleware};
use r2d2::{Config, Pool};
use r2d2_postgres::{TlsMode, PostgresConnectionManager};

use std::sync::Arc;

use errors::*;

pub struct DatabaseProvider {
    pool: Arc<Pool<PostgresConnectionManager>>
}

impl typemap::Key for DatabaseProvider {
    type Value = Arc<Pool<PostgresConnectionManager>>;
}

impl DatabaseProvider {
    pub fn new(config: &::config::Config) -> Result<DatabaseProvider> {
        config.get_table("postgresql")
            .ok_or_else(|| Error::from(ErrorKind::MissingConfigValue("postgresql".to_string())))
            .and_then(|table| table.get("url")
                .and_then(|i| i.clone().into_str())
                .ok_or_else(|| Error::from(ErrorKind::MissingConfigValueTable("url".to_string(), "postgresql".to_string()))))
            .and_then(|url| {
                let r2d2_config = Config::default();
                /*let r2d2_config = config.get_table("postgresql")
                    .and_then(|table| table.get("pool")
                        .and_then(|p| p.clone().into_table()))
                    .and_then(|pool| {
                        let builder = Config::builder();
                        pool.get("pool_size")
                            .and_then(|p| p.clone().into_int())
                            .map(|size| {
                                let builder = builder.pool_size(size as u32)
                                    .min_idle(pool.get("min_idle")
                                        .and_then(|i| i.clone().into_int())
                                        .map(|i| i as u32));
                                pool.get("helper_threads")
                                    .and_then(|h| h.clone().into_int())
                                    .map(|threads| builder.helper_threads(threads as u32));
                                pool.get("test_on_check_out")
                                    .and_then(|t| t.clone().into_bool())
                                    .map(|test| builder.test_on_check_out(test));
                                pool.get("init_fail_fast")
                                    .and_then(|i| i.clone().into_bool())
                                    .map(|init| builder.initialization_fail_fast(init));
                                builder.max_lifetime(pool.get("max_lifetime").and_then(|m| m.clone().into_int())
                                    .map(|max| Duration::from_secs(max as u64)));
                                builder.idle_timeout(pool.get("idle_timeout").and_then(|i| i.clone().into_int())
                                    .map(|idle| Duration::from_secs(idle as u64)));
                                pool.get("connection_timeout")
                                    .and_then(|c| c.clone().into_int())
                                    .map(|time| builder.connection_timeout(Duration::from_secs(time as u64)));
                                builder.build()
                            })
                    }).unwrap_or_else(Config::default);*/
                PostgresConnectionManager::new(url, TlsMode::None)
                    .map_err(|err| Error::from(ErrorKind::PostgresConnect(err)))
                    .and_then(|manager| Pool::new(r2d2_config, manager)
                        .map_err(|err| Error::from(ErrorKind::PoolInitialisation(err))))
            }).map(|pool| DatabaseProvider {pool: Arc::new(pool)})
    }
}

impl BeforeMiddleware for DatabaseProvider {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<DatabaseProvider>(self.pool.clone());
        Ok(())
    }
}