---
title: "Alpha Vantage API Documentation"
source: "https://www.alphavantage.co/documentation"
extracted_at: "2024-01-07T12:00:00Z"
endpoint_count: 2
category_count: 1
format_version: "2.0"
---

## Table of Contents

- [Stock Time Series Data](#stock-time-series-data)
  - [TIME_SERIES_INTRADAY](#time-series-intraday)

## Stock Time Series Data

This category contains endpoints for retrieving historical stock price data with various intervals and formats.

### TIME_SERIES_INTRADAY

Returns intraday time series data for a given stock symbol with 1-minute, 5-minute, 15-minute, 30-minute, or 60-minute intervals.

**Required Parameters:**

| Parameter | Type | Description | Default | Options |
|-----------|------|-------------|---------|---------|
| function | string | The API function to call | - | - |
| symbol | string | The stock symbol to retrieve data for | - | - |
| apikey | string | Your API key | - | - |

**Optional Parameters:**

| Parameter | Type | Description | Default | Options |
|-----------|------|-------------|---------|---------|
| interval | string | Time interval between data points | 5min | 1min, 5min, 15min, 30min, 60min |
| outputsize | string | The size of the output | compact | compact, full |

```python
import requests

url = 'https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY&symbol=IBM&interval=5min&apikey=demo'
r = requests.get(url)
data = r.json()
print(data)
```

**API Request Pattern:**

```http
https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY&symbol={symbol}&interval={interval}&outputsize={outputsize}&apikey={apikey}
```
