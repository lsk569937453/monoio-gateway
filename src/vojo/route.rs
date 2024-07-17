use super::app_error::AppError;

use core::fmt::Debug;
use http::HeaderMap;
use http::HeaderValue;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use tracing::metadata::LevelFilter;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::enum_variant_names)]
#[serde(tag = "type")]
pub enum LoadbalancerStrategy {
    PollRoute(PollRoute),
    HeaderRoute(HeaderRoute),
    RandomRoute(RandomRoute),
    WeightRoute(WeightRoute),
}

impl LoadbalancerStrategy {
    pub async fn get_route(
        &mut self,
        headers: HeaderMap<HeaderValue>,
    ) -> Result<BaseRoute, AppError> {
        match self {
            LoadbalancerStrategy::PollRoute(poll_route) => poll_route.get_route(headers).await,

            LoadbalancerStrategy::HeaderRoute(poll_route) => poll_route.get_route(headers).await,

            LoadbalancerStrategy::RandomRoute(poll_route) => poll_route.get_route(headers).await,

            LoadbalancerStrategy::WeightRoute(poll_route) => poll_route.get_route(headers).await,
        }
    }
    pub fn get_all_route(&mut self) -> Result<Vec<&mut BaseRoute>, AppError> {
        match self {
            LoadbalancerStrategy::PollRoute(poll_route) => poll_route.get_all_route(),
            LoadbalancerStrategy::HeaderRoute(poll_route) => poll_route.get_all_route(),

            LoadbalancerStrategy::RandomRoute(poll_route) => poll_route.get_all_route(),

            LoadbalancerStrategy::WeightRoute(poll_route) => poll_route.get_all_route(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AnomalyDetectionStatus {
    pub consecutive_5xx: i32,
}
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Serialize)]
pub struct BaseRoute {
    pub endpoint: String,
    pub try_file: Option<String>,
    #[serde(default = "default_base_route_id")]
    pub base_route_id: String,
    #[serde(skip_deserializing)]
    pub is_alive: Option<bool>,
    #[serde(skip_serializing, skip_deserializing)]
    pub anomaly_detection_status: AnomalyDetectionStatus,
}
fn default_base_route_id() -> String {
    let id = Uuid::new_v4();
    id.to_string()
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WeightRouteNestedItem {
    pub base_route: BaseRoute,
    pub weight: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SplitSegment {
    pub split_by: String,
    pub split_list: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SplitItem {
    pub header_key: String,
    pub header_value: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub struct RegexMatch {
    pub value: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextMatch {
    pub value: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HeaderValueMappingType {
    Regex(RegexMatch),
    Text(TextMatch),
    Split(SplitSegment),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeaderRouteNestedItem {
    pub base_route: BaseRoute,
    pub header_key: String,
    pub header_value_mapping_type: HeaderValueMappingType,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HeaderRoute {
    pub routes: Vec<HeaderRouteNestedItem>,
}

impl HeaderRoute {
    fn get_all_route(&mut self) -> Result<Vec<&mut BaseRoute>, AppError> {
        let vecs = self
            .routes
            .iter_mut()
            .map(|item| &mut item.base_route)
            .collect::<Vec<&mut BaseRoute>>();
        Ok(vecs)
    }

    async fn get_route(&mut self, headers: HeaderMap<HeaderValue>) -> Result<BaseRoute, AppError> {
        let mut alive_cluster: Vec<HeaderRouteNestedItem> = vec![];
        for item in self.routes.clone() {
            let is_alve_result = item.base_route.is_alive;
            let is_alive = is_alve_result.unwrap_or(true);
            if is_alive {
                alive_cluster.push(item.clone());
            }
        }
        for item in alive_cluster.iter() {
            let headers_contais_key = headers.contains_key(item.header_key.clone());
            if !headers_contais_key {
                continue;
            }
            let header_value = headers.get(item.header_key.clone()).unwrap();
            let header_value_str = header_value.to_str().unwrap();
            match item.clone().header_value_mapping_type {
                HeaderValueMappingType::Regex(regex_str) => {
                    let re = Regex::new(&regex_str.value).unwrap();
                    let capture_option = re.captures(header_value_str);
                    if capture_option.is_none() {
                        continue;
                    } else {
                        return Ok(item.clone().base_route);
                    }
                }
                HeaderValueMappingType::Text(text_str) => {
                    if text_str.value == header_value_str {
                        return Ok(item.clone().base_route);
                    } else {
                        continue;
                    }
                }
                HeaderValueMappingType::Split(split_segment) => {
                    let split_set: HashSet<_> =
                        header_value_str.split(&split_segment.split_by).collect();
                    if split_set.is_empty() {
                        continue;
                    }
                    let mut flag = true;
                    for split_item in split_segment.split_list.iter() {
                        if !split_set.contains(split_item.clone().as_str()) {
                            flag = false;
                            break;
                        }
                    }
                    if flag {
                        return Ok(item.clone().base_route);
                    }
                }
            }
        }
        error!("Can not find the route!And siverWind has selected the first route!");

        let first = alive_cluster.first().unwrap().base_route.clone();
        Ok(first)
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RandomBaseRoute {
    pub base_route: BaseRoute,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RandomRoute {
    pub routes: Vec<RandomBaseRoute>,
}

impl RandomRoute {
    fn get_all_route(&mut self) -> Result<Vec<&mut BaseRoute>, AppError> {
        let vecs = self
            .routes
            .iter_mut()
            .map(|item| &mut item.base_route)
            .collect::<Vec<&mut BaseRoute>>();
        Ok(vecs)
    }

    async fn get_route(&mut self, _headers: HeaderMap<HeaderValue>) -> Result<BaseRoute, AppError> {
        let mut alive_cluster: Vec<BaseRoute> = vec![];
        for item in self.routes.clone() {
            let is_alve_result = item.base_route.is_alive;
            let is_alive = is_alve_result.unwrap_or(true);
            if is_alive {
                alive_cluster.push(item.base_route.clone());
            }
        }
        let mut rng = thread_rng();
        let index = rng.gen_range(0..alive_cluster.len());
        let dst = alive_cluster[index].clone();
        Ok(dst)
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PollBaseRoute {
    pub base_route: BaseRoute,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PollRoute {
    #[serde(skip)]
    pub current_index: i64,
    pub routes: Vec<PollBaseRoute>,
}

impl PollRoute {
    fn get_all_route(&mut self) -> Result<Vec<&mut BaseRoute>, AppError> {
        let vecs = self
            .routes
            .iter_mut()
            .map(|item| &mut item.base_route)
            .collect::<Vec<&mut BaseRoute>>();
        Ok(vecs)
    }

    async fn get_route(&mut self, _headers: HeaderMap<HeaderValue>) -> Result<BaseRoute, AppError> {
        let mut alive_cluster: Vec<PollBaseRoute> = vec![];
        for item in self.routes.clone() {
            let is_alve_result = item.base_route.is_alive;
            let is_alive = is_alve_result.unwrap_or(true);
            if is_alive {
                alive_cluster.push(item.clone());
            }
        }
        if alive_cluster.is_empty() {
            return Err(AppError(String::from(
                "Can not find alive host in the clusters",
            )));
        }
        let older = self.current_index;
        let len = alive_cluster.len();
        let current_index = (older + 1) % len as i64;
        self.current_index = current_index;
        let dst = alive_cluster[current_index as usize].clone();
        let level_filter = tracing_subscriber::filter::LevelFilter::current();

        if level_filter == LevelFilter::DEBUG {
            debug!(
                "PollRoute current index:{},cluter len:{},older index:{}",
                current_index as i32, len, older
            );
        }
        Ok(dst.base_route)
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WeightRoute {
    pub routes: Vec<WeightRouteNestedItem>,
    pub index: u64,
    pub offset: u64,
}

impl WeightRoute {
    fn get_all_route(&mut self) -> Result<Vec<&mut BaseRoute>, AppError> {
        let vecs = self
            .routes
            .iter_mut()
            .map(|item| &mut item.base_route)
            .collect::<Vec<&mut BaseRoute>>();
        Ok(vecs)
    }

    async fn get_route(&mut self, _headers: HeaderMap<HeaderValue>) -> Result<BaseRoute, AppError> {
        let cluster_read_lock2 = self.routes.clone();
        loop {
            let currnet_index = self.index;
            let offset = self.offset;
            let current_weight = cluster_read_lock2
                .get(currnet_index as usize)
                .ok_or(AppError(String::from("")))?;
            let is_alive = current_weight.base_route.is_alive.unwrap_or(true);
            if current_weight.weight > offset && is_alive {
                self.offset += 1;
                return Ok(current_weight.base_route.clone());
            }
            if current_weight.weight <= offset {
                self.offset = 0;
                self.index = (self.index + 1) % cluster_read_lock2.len() as u64;
                continue;
            }
            if !is_alive {
                self.offset = 0;
                self.index = (self.index + 1) % cluster_read_lock2.len() as u64;
                continue;
            }
        }

        // Err(AppError(String::from("WeightRoute get route error")))
    }
}
