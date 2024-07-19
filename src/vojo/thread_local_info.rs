pub struct ThreadLocalInfo {
    pub thread_local_route_info: ThreadLocalRouteInfo,
}
pub struct ThreadLocalRouteInfo {
    pub round_robin_info: RoundRobinRouterInfo,
    pub weighted_info: WeightedRouterInfo,
}
pub struct RoundRobinRouterInfo {
    pub route_index: i32,
}
pub struct WeightedRouterInfo {
    pub index: i32,
    pub off_set: i32,
}
