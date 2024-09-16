use anyhow::Context;
use async_stream::stream;
use futures::Stream;
use leaky_bucket::RateLimiter;
use octocrab::models::Repository;
use std::{collections::HashMap, default::Default, io::Read, time::Duration};
use subprocess::Exec;
use tracing::{debug, info};

pub struct Connection {
    auth_token: Option<String>,
    rate_limiters: HashMap<bool, Vec<RateLimiter>>, // key is whether we have an auth token
}

impl Connection {
    pub fn new() -> Self {
        let mut this = Self {
            auth_token: None,
            // https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api
            rate_limiters: HashMap::from([
                (
                    false, // unauthenticated
                    vec![
                        // unauthenticated primary
                        RateLimiter::builder()
                            .fair(false)
                            .max(60)
                            .initial(60)
                            .interval(Duration::from_secs(60 * 60)) // 1 hour
                            .refill(60)
                            .build(),
                        // unauthenticated secondary REST GET, HEAD, OPTIONS
                        RateLimiter::builder()
                            .fair(false)
                            .max(900)
                            .initial(900)
                            .interval(Duration::from_secs(60)) // 1 min
                            .refill(900)
                            .build(),
                    ],
                ),
                (
                    true, // authenticated
                    vec![
                        // authenticated primary
                        RateLimiter::builder()
                            .fair(false)
                            .max(5000)
                            .initial(5000)
                            .interval(Duration::from_secs(60 * 60)) // 1 hour
                            .refill(5000)
                            .build(),
                        // authenticated secondary REST GET, HEAD, OPTIONS
                        RateLimiter::builder()
                            .fair(false)
                            .max(900)
                            .initial(900)
                            .interval(Duration::from_secs(60)) // 1 min
                            .refill(900)
                            .build(),
                    ],
                ),
            ]),
        };

        this.try_auth();

        this
    }

    fn try_auth(&mut self) {
        match Exec::cmd("gh")
            .args(&["auth", "token"])
            .stream_stdout()
            .context("gh auth token")
        {
            Ok(mut stream) => {
                let mut token = String::new();
                stream.read_to_string(&mut token).unwrap();
                debug!("acquired GitHub token");
                self.auth_token = Some(token);
            }
            Err(e) => {
                debug!("failed to acquired GitHub token: {}", e);
            }
        }
    }

    async fn rate_limit_one(&self) {
        let authorized = self.auth_token.is_some();
        for (i, rate_limiter) in self.rate_limiters[&authorized].iter().enumerate() {
            if !rate_limiter.try_acquire(1) {
                info!(
                    "rate limited {} auth by {} {:?}, please wait",
                    if authorized { "with" } else { "without" },
                    i,
                    &rate_limiter
                );
                if !authorized {
                    info!("login to GitHub for higher rate");
                }
                rate_limiter.acquire_one().await
            }
        }
    }

    pub fn git_dirs<'a, 'b, S>(&'a self, user: S) -> impl Stream<Item = String> + 'b
    where
        S: Into<String>,
        'a: 'b,
    {
        let user: String = user.into();
        stream! {
            let octocrab = octocrab::instance();
            let user = octocrab.users(user);
            let repos = user.repos();

            self.rate_limit_one().await;
            let mut page = Some(repos.send().await.unwrap());

            while let Some(mut current_page) = page {
            for repo in current_page.take_items() {
                yield repo.name;
            }

            if current_page.next.is_some() {
                self.rate_limit_one().await;
            }
                page = octocrab.get_page::<Repository>(&current_page.next).await.unwrap();
            }
        }
    }
}

impl Default for Connection {
    fn default() -> Self {
        Self::new()
    }
}
