use super::{Duration, Epoch};
/*

NOTE: This is taken from itertools: https://docs.rs/itertools-num/0.1.3/src/itertools_num/linspace.rs.html#78-93 .

*/

/// An iterator of a sequence of evenly spaced Epochs.
#[derive(Clone, Debug)]
pub struct TimeSeries {
    start: Epoch,
    end: Epoch,
    step: Duration,
    cur: Epoch,
    incl: bool,
}

impl TimeSeries {
    /// Return an iterator of evenly spaced Epochs, **inclusive** on start and **exclusive** on end.
    /// ```
    /// use hifitime::{Epoch, Unit, TimeSeries};
    /// let start = Epoch::from_gregorian_utc_at_midnight(2017, 1, 14);
    /// let end = Epoch::from_gregorian_utc_at_noon(2017, 1, 14);
    /// let step = Unit::Hour * 2;
    /// let time_series = TimeSeries::exclusive(start, end, step);
    /// let mut cnt = 0;
    /// for epoch in time_series {
    ///     println!("{}", epoch);
    ///     cnt += 1
    /// }
    /// assert_eq!(cnt, 6)
    /// ```
    #[inline]
    pub fn exclusive(start: Epoch, end: Epoch, step: Duration) -> TimeSeries {
        // Start one step prior to start because next() just moves forward
        Self {
            start,
            end,
            step,
            cur: start - step,
            incl: false,
        }
    }

    /// Return an iterator of evenly spaced Epochs, inclusive on start **and** on end.
    /// ```
    /// use hifitime::{Epoch, Unit, TimeSeries};
    /// let start = Epoch::from_gregorian_utc_at_midnight(2017, 1, 14);
    /// let end = Epoch::from_gregorian_utc_at_noon(2017, 1, 14);
    /// let step = Unit::Hour * 2;
    /// let time_series = TimeSeries::inclusive(start, end, step);
    /// let mut cnt = 0;
    /// for epoch in time_series {
    ///     println!("{}", epoch);
    ///     cnt += 1
    /// }
    /// assert_eq!(cnt, 7)
    /// ```
    #[inline]
    pub fn inclusive(start: Epoch, end: Epoch, step: Duration) -> TimeSeries {
        // Start one step prior to start because next() just moves forward
        Self {
            start,
            end,
            step,
            cur: start - step,
            incl: true,
        }
    }
}

impl Iterator for TimeSeries {
    type Item = Epoch;

    #[inline]
    fn next(&mut self) -> Option<Epoch> {
        let next_item = self.cur + self.step;
        if (!self.incl && next_item >= self.end) || (self.incl && next_item > self.end) {
            None
        } else {
            self.cur = next_item;
            Some(next_item)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len() + 1))
    }
}

impl DoubleEndedIterator for TimeSeries {
    #[inline]
    fn next_back(&mut self) -> Option<Epoch> {
        let next_item = self.cur - self.step;
        if next_item < self.start {
            None
        } else {
            Some(next_item)
        }
    }
}

impl ExactSizeIterator for TimeSeries
where
    TimeSeries: Iterator,
{
    fn len(&self) -> usize {
        let approx = ((self.end - self.start).in_seconds() / self.step.in_seconds()).abs();
        if self.incl {
            if approx.ceil() >= usize::MAX as f64 {
                usize::MAX
            } else {
                approx.ceil() as usize
            }
        } else {
            if approx.floor() >= usize::MAX as f64 {
                usize::MAX
            } else {
                approx.floor() as usize
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Epoch, TimeSeries, Unit};

    #[test]
    fn test_timeseries() {
        let start = Epoch::from_gregorian_utc_at_midnight(2017, 1, 14);
        let end = Epoch::from_gregorian_utc_at_noon(2017, 1, 14);
        let step = Unit::Hour * 2;

        let mut count = 0;
        let time_series = TimeSeries::exclusive(start, end, step);
        for epoch in time_series {
            if count == 0 {
                assert_eq!(
                    epoch, start,
                    "Starting epoch of exclusive time series is wrong"
                );
            } else if count == 5 {
                assert_ne!(epoch, end, "Ending epoch of exclusive time series is wrong");
            }
            #[cfg(feature = "std")]
            println!("{}", epoch);
            count += 1;
        }

        assert_eq!(count, 6, "Should have five items in this iterator");

        count = 0;
        let time_series = TimeSeries::inclusive(start, end, step);
        for epoch in time_series {
            if count == 0 {
                assert_eq!(
                    epoch, start,
                    "Starting epoch of inclusive time series is wrong"
                );
            } else if count == 6 {
                assert_eq!(epoch, end, "Ending epoch of inclusive time series is wrong");
            }
            #[cfg(feature = "std")]
            println!("{}", epoch);
            count += 1;
        }

        assert_eq!(count, 7, "Should have six items in this iterator");
    }

    #[test]
    fn gh131_regression() {
        let start = Epoch::from_gregorian_str("2022-07-14T02:56:11.228271007 UTC").unwrap();
        let step = 0.5 * Unit::Microsecond;
        let steps = 1_000_000_000;
        let end = start + steps * step; // This is 500 ms later
        let times = TimeSeries::exclusive(start, end, step);
        // For an _exclusive_ time series, we skip the last item, so it's steps minus one
        assert_eq!(times.len(), steps as usize - 1);
        assert_eq!(times.len(), times.size_hint().0);

        let times = TimeSeries::inclusive(start, end, step);
        // For an _exclusive_ time series, we skip the last item, so it's the steps count
        assert_eq!(times.len(), steps as usize);
        assert_eq!(times.len(), times.size_hint().0);
    }
}
