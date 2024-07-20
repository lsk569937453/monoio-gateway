pub struct ThreadLocalInfo {
    pub thread_local_route_info: ThreadLocalRouteInfo,
}
impl ThreadLocalInfo {
    pub fn new() -> Self {
        ThreadLocalInfo {
            thread_local_route_info: ThreadLocalRouteInfo {
                round_robin_info: RoundRobinRouterInfo { route_index: 0 },
                weighted_info: WeightedRouterInfo {
                    index: 0,
                    off_set: 0,
                },
            },
        }
    }
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
