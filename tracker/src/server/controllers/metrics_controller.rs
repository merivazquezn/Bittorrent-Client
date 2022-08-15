use crate::http::HttpGetRequest;
use crate::http::IHttpService;
use crate::metrics::GroupBy;
use crate::metrics::MetricsSender;
use crate::metrics::TimeFrame;
use crate::server::errors::MetricsError;
use crate::server::constants::*;

pub struct MetricsController;

impl MetricsController {
    pub fn handler_metrics(
        mut http_service: Box<dyn IHttpService>,
        request: HttpGetRequest,
        metrics: MetricsSender,
    ) -> Result<(), MetricsError> {
        // get key param from request, otherwise throw error MetricsError::MissingKey
        let metric_key = request
            .params
            .get(METRIC_KEY)
            .ok_or_else(||MetricsError::MissingKey(METRIC_KEY.to_owned()))?;
        let time_frame_key = request
            .params
            .get(TIME_FRAME_INTERVAL_KEY)
            .ok_or_else(||MetricsError::MissingKey(TIME_FRAME_INTERVAL_KEY.to_owned()))?;
        let time_frame_count_key = request
            .params
            .get(TIME_FRAME_COUNT_KEY)
            .ok_or_else(||MetricsError::MissingKey(TIME_FRAME_COUNT_KEY.to_owned()))?;
        let groupby_key = request
            .params
            .get(GROUP_BY_KEY)
            .ok_or_else(||MetricsError::MissingKey(GROUP_BY_KEY.to_owned()))?;
        let groupby_count_key = request
            .params
            .get(GROUP_BY_COUNT_KEY)
            .ok_or_else(||MetricsError::MissingKey(GROUP_BY_COUNT_KEY.to_owned()))?;

        let time_frame = Self::get_requested_time_frame(time_frame_count_key,time_frame_key)?;
        let groupby = Self::get_requested_groupby(groupby_count_key,groupby_key)?;
        let json: String = metrics.get_metrics_response(
            metric_key.to_owned(),
            time_frame,
            groupby,
        )?;

        http_service.send_ok_response(json.as_bytes().to_vec(), "application/json".to_string())?;

        Ok(())
    }
    
    fn get_requested_time_frame(time_frame_count_key: &str, time_frame_key: &str ) -> Result<TimeFrame, MetricsError>{
        let time_frame_count: u32 = time_frame_count_key.parse()?;
        match time_frame_key {
            TIME_FRAME_DAYS_KEY => Ok(TimeFrame::LastDays(time_frame_count)),
            TIME_FRAME_HOURS_KEY => Ok(TimeFrame::LastHours(time_frame_count)),
            &_ => Err(MetricsError::InvalidTimeFrameReceived(time_frame_key.to_string()))
        }
    }

    fn get_requested_groupby(groupby_count_key: &str, groupby_key: &str ) -> Result<GroupBy, MetricsError>{
        let groupby_count: u32 = groupby_count_key.parse()?;
        match groupby_key {
            GROUPBY_HOURS_KEY => Ok(GroupBy::Hours(groupby_count)),
            GROUPBY_MINUTES_KEY => Ok(GroupBy::Minutes(groupby_count)),
            &_ => Err(MetricsError::InvalidGroupByReceived(groupby_key.to_string()))
        }
        
    }
}
