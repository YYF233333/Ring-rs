(function () {
  'use strict';

  // ── Formatting helpers ──

  function fmtBytes(b) {
    if (b >= 1e9) return (b / 1e9).toFixed(2) + ' GB';
    if (b >= 1e6) return (b / 1e6).toFixed(2) + ' MB';
    if (b >= 1e3) return (b / 1e3).toFixed(1) + ' KB';
    return b + ' B';
  }

  function fmtSecs(s) {
    if (s >= 60) {
      var m = Math.floor(s / 60);
      var sec = (s % 60).toFixed(0);
      return m + 'm ' + sec + 's';
    }
    return s.toFixed(1) + 's';
  }

  function fmtPct(v) { return v.toFixed(1) + '%'; }
  function fmtNum(v) { return v.toLocaleString(); }

  // ── Shared chart config ──

  var COLORS = {
    blue: '#4fc3f7',
    green: '#66bb6a',
    orange: '#ffa726',
    red: '#ef5350',
    purple: '#ab47bc',
    cyan: '#26c6da',
    lime: '#9ccc65',
    pink: '#ec407a',
  };

  var AXIS_LABEL = { color: '#8888aa', fontSize: 11 };
  var AXIS_LINE = { lineStyle: { color: '#2a2a4a' } };
  var SPLIT_LINE = { lineStyle: { color: '#1e1e3a', type: 'dashed' } };

  function baseXAxis() {
    return {
      type: 'time',
      axisLabel: AXIS_LABEL,
      axisLine: AXIS_LINE,
      axisTick: { show: false },
    };
  }

  function baseYAxis(name, formatter) {
    return {
      type: 'value',
      name: name,
      nameTextStyle: { color: '#8888aa', fontSize: 11 },
      axisLabel: Object.assign({}, AXIS_LABEL, formatter ? { formatter: formatter } : {}),
      axisLine: { show: false },
      splitLine: SPLIT_LINE,
    };
  }

  function tooltipFormatter(data, valueFmt) {
    return function (params) {
      var idx = params[0].dataIndex;
      var commit = data[idx].commit_short || '';
      var d = new Date(params[0].value[0]);
      var dateStr = d.getFullYear() + '-' +
        String(d.getMonth() + 1).padStart(2, '0') + '-' +
        String(d.getDate()).padStart(2, '0') + ' ' +
        String(d.getHours()).padStart(2, '0') + ':' +
        String(d.getMinutes()).padStart(2, '0');
      var header = '<div style="margin-bottom:6px;font-weight:600">' +
        dateStr + ' <span style="color:#8888aa">(' + commit + ')</span></div>';
      var rows = params.map(function (p) {
        var val = valueFmt ? valueFmt(p.value[1]) : p.value[1];
        return '<span style="display:inline-block;width:8px;height:8px;border-radius:50%;background:' +
          p.color + ';margin-right:6px"></span>' + p.seriesName + ': <strong>' + val + '</strong>';
      }).join('<br/>');
      return header + rows;
    };
  }

  function baseTooltip(data, valueFmt) {
    return {
      trigger: 'axis',
      backgroundColor: '#1a1a2e',
      borderColor: '#2a2a4a',
      textStyle: { color: '#e0e0e0', fontSize: 12 },
      formatter: tooltipFormatter(data, valueFmt),
    };
  }

  function baseLegend() {
    return {
      textStyle: { color: '#8888aa', fontSize: 11 },
      top: 4,
      right: 16,
      itemWidth: 14,
      itemHeight: 8,
    };
  }

  function baseGrid() {
    return { left: 60, right: 24, top: 48, bottom: 36 };
  }

  function dataZoom() {
    return [{ type: 'inside', start: 0, end: 100 }];
  }

  function timeSeries(name, dates, values, color, opts) {
    var paired = dates.map(function (d, i) { return [d, values[i]]; });
    return Object.assign({
      name: name,
      type: 'line',
      data: paired,
      symbol: 'circle',
      symbolSize: 4,
      lineStyle: { width: 2, color: color },
      itemStyle: { color: color },
      emphasis: { focus: 'series' },
    }, opts || {});
  }

  // ── Init chart helper ──

  function initChart(id) {
    var dom = document.getElementById(id);
    return echarts.init(dom, null, { renderer: 'canvas' });
  }

  // ── Summary cards ──

  function renderCards(latest, prev) {
    var container = document.getElementById('summary-cards');
    var m = latest.metrics;

    var cards = [
      { label: 'Coverage', value: fmtPct(m.coverage.workspace), prev: prev && prev.metrics.coverage.workspace, unit: '%', higherBetter: true },
      { label: 'Binary', value: fmtBytes(m.binary_size_bytes.host_release), prev: prev && prev.metrics.binary_size_bytes.host_release, unit: '', higherBetter: false, raw: m.binary_size_bytes.host_release },
      { label: 'Release Build', value: fmtSecs(m.build_time_secs.release_clean), prev: prev && prev.metrics.build_time_secs.release_clean, unit: '', higherBetter: false, raw: m.build_time_secs.release_clean },
      { label: 'Debug Build', value: fmtSecs(m.build_time_secs.debug_clean), prev: prev && prev.metrics.build_time_secs.debug_clean, unit: '', higherBetter: false, raw: m.build_time_secs.debug_clean },
      { label: 'Effective LOC', value: fmtNum(m.effective_loc), prev: prev && prev.metrics.effective_loc, unit: '', higherBetter: null },
      { label: 'Dependencies', value: fmtNum(m.dependencies.transitive), prev: prev && prev.metrics.dependencies.transitive, unit: '', higherBetter: false },
      { label: 'Tests', value: fmtNum(m.tests.passed), prev: prev && prev.metrics.tests.passed, unit: '', higherBetter: true },
    ];

    container.innerHTML = cards.map(function (c) {
      var deltaHtml = '';
      if (c.prev != null) {
        var curVal = c.raw != null ? c.raw : parseFloat(c.value.replace(/[^0-9.]/g, ''));
        var prevVal = c.prev;
        var diff = curVal - prevVal;
        if (diff !== 0) {
          var cls = c.higherBetter === null ? 'neutral' : ((diff > 0) === c.higherBetter ? 'up' : 'down');
          var sign = diff > 0 ? '+' : '';
          var display;
          if (c.label === 'Binary') display = sign + fmtBytes(Math.abs(diff));
          else if (c.label.includes('Build')) display = sign + diff.toFixed(1) + 's';
          else if (c.label === 'Coverage') display = sign + diff.toFixed(1) + '%';
          else display = sign + fmtNum(diff);
          deltaHtml = '<div class="delta ' + cls + '">' + display + '</div>';
        }
      }
      return '<div class="card"><div class="label">' + c.label + '</div><div class="value">' + c.value + '</div>' + deltaHtml + '</div>';
    }).join('');
  }

  // ── Chart builders ──

  function chartCoverage(data, dates) {
    var chart = initChart('chart-coverage');
    chart.setOption({
      tooltip: baseTooltip(data, function (v) { return v + '%'; }),
      legend: baseLegend(),
      grid: baseGrid(),
      dataZoom: dataZoom(),
      xAxis: baseXAxis(),
      yAxis: baseYAxis('%', function (v) { return v + '%'; }),
      series: [
        timeSeries('workspace', dates, data.map(function (d) { return d.metrics.coverage.workspace; }), COLORS.blue),
        timeSeries('vn-runtime', dates, data.map(function (d) { return d.metrics.coverage.vn_runtime; }), COLORS.green),
        timeSeries('host', dates, data.map(function (d) { return d.metrics.coverage.host; }), COLORS.orange),
      ],
    });
    return chart;
  }

  function chartBinary(data, dates) {
    var chart = initChart('chart-binary');
    chart.setOption({
      tooltip: baseTooltip(data, fmtBytes),
      legend: baseLegend(),
      grid: baseGrid(),
      dataZoom: dataZoom(),
      xAxis: baseXAxis(),
      yAxis: baseYAxis('Size', function (v) { return fmtBytes(v); }),
      series: [
        timeSeries('release', dates, data.map(function (d) { return d.metrics.binary_size_bytes.host_release; }), COLORS.blue),
      ],
    });
    return chart;
  }

  function chartBuildTime(data, dates) {
    var chart = initChart('chart-build-time');
    chart.setOption({
      tooltip: baseTooltip(data, fmtSecs),
      legend: baseLegend(),
      grid: baseGrid(),
      dataZoom: dataZoom(),
      xAxis: baseXAxis(),
      yAxis: baseYAxis('Time', function (v) { return fmtSecs(v); }),
      series: [
        timeSeries('release', dates, data.map(function (d) { return d.metrics.build_time_secs.release_clean; }), COLORS.red),
        timeSeries('debug', dates, data.map(function (d) { return d.metrics.build_time_secs.debug_clean; }), COLORS.orange),
      ],
    });
    return chart;
  }

  function chartLoc(data, dates) {
    var chart = initChart('chart-loc');
    chart.setOption({
      tooltip: baseTooltip(data),
      legend: baseLegend(),
      grid: baseGrid(),
      dataZoom: dataZoom(),
      xAxis: baseXAxis(),
      yAxis: baseYAxis('Lines'),
      series: [
        timeSeries('effective LOC', dates, data.map(function (d) { return d.metrics.effective_loc; }), COLORS.cyan, { areaStyle: { color: 'rgba(38,198,218,0.08)' } }),
        timeSeries('vn-runtime (code)', dates, data.map(function (d) { return d.metrics.tokei_rust.vn_runtime.code; }), COLORS.green),
        timeSeries('host (code)', dates, data.map(function (d) { return d.metrics.tokei_rust.host.code; }), COLORS.blue),
        timeSeries('xtask (code)', dates, data.map(function (d) { return d.metrics.tokei_rust.xtask.code; }), COLORS.purple),
      ],
    });
    return chart;
  }

  function chartDeps(data, dates) {
    var chart = initChart('chart-deps');
    chart.setOption({
      tooltip: baseTooltip(data),
      legend: baseLegend(),
      grid: baseGrid(),
      dataZoom: dataZoom(),
      xAxis: baseXAxis(),
      yAxis: baseYAxis('Count'),
      series: [
        timeSeries('transitive', dates, data.map(function (d) { return d.metrics.dependencies.transitive; }), COLORS.orange),
        timeSeries('direct', dates, data.map(function (d) { return d.metrics.dependencies.direct; }), COLORS.blue),
      ],
    });
    return chart;
  }

  function chartTests(data, dates) {
    var chart = initChart('chart-tests');
    chart.setOption({
      tooltip: baseTooltip(data),
      legend: baseLegend(),
      grid: baseGrid(),
      dataZoom: dataZoom(),
      xAxis: baseXAxis(),
      yAxis: baseYAxis('Count'),
      series: [
        timeSeries('passed', dates, data.map(function (d) { return d.metrics.tests.passed; }), COLORS.green),
        timeSeries('ignored', dates, data.map(function (d) { return d.metrics.tests.ignored; }), COLORS.orange),
        timeSeries('failed', dates, data.map(function (d) { return d.metrics.tests.failed; }), COLORS.red),
      ],
    });
    return chart;
  }

  // ── Main ──

  fetch('data.json')
    .then(function (r) { return r.json(); })
    .then(function (data) {
      if (!Array.isArray(data)) data = [data];
      if (!data || data.length === 0) {
        document.getElementById('summary-cards').innerHTML =
          '<div class="card" style="grid-column:1/-1"><div class="label">No data yet</div><div class="value">Waiting for first CI run</div></div>';
        return;
      }

      var dates = data.map(function (d) { return new Date(d.date).getTime(); });
      var latest = data[data.length - 1];
      var prev = data.length > 1 ? data[data.length - 2] : null;

      renderCards(latest, prev);

      var charts = [
        chartCoverage(data, dates),
        chartBinary(data, dates),
        chartBuildTime(data, dates),
        chartLoc(data, dates),
        chartDeps(data, dates),
        chartTests(data, dates),
      ];

      window.addEventListener('resize', function () {
        charts.forEach(function (c) { c.resize(); });
      });
    })
    .catch(function (err) {
      console.error('Failed to load metrics data:', err);
      document.getElementById('summary-cards').innerHTML =
        '<div class="card" style="grid-column:1/-1"><div class="label">Error</div><div class="value">Failed to load data.json</div></div>';
    });
})();
