import type { ChartResponse } from "@healthii/types";

export function Sparkline({ chart }: { chart: ChartResponse }) {
  const width = 560;
  const height = 160;
  const pad = 16;
  if (chart.points.length === 0) {
    return <p className="hii-hint">No points yet for this metric.</p>;
  }
  const xs = chart.points.map((point) => new Date(point.t).getTime());
  const ys = chart.points.map((point) => point.v);
  const minX = Math.min(...xs);
  const maxX = Math.max(...xs);
  const minY = chart.min ?? Math.min(...ys);
  const maxY = chart.max ?? Math.max(...ys);
  const spanX = Math.max(maxX - minX, 1);
  const spanY = Math.max(maxY - minY, 0.0001);
  const coords = chart.points.map((point) => {
    const x = pad + ((new Date(point.t).getTime() - minX) / spanX) * (width - pad * 2);
    const y = height - pad - ((point.v - minY) / spanY) * (height - pad * 2);
    return { x, y, v: point.v };
  });
  const line = coords.map((point) => `${point.x},${point.y}`).join(" ");
  const area = `${coords[0].x},${height - pad} ${line} ${coords[coords.length - 1].x},${height - pad}`;
  const last = coords[coords.length - 1];

  return (
    <div>
      <svg className="hii-chart" viewBox={`0 0 ${width} ${height}`} role="img" aria-label={`${chart.metric} chart`}>
        <polygon className="hii-chart-fill" points={area} />
        <polyline
          className="hii-chart-line"
          fill="none"
          strokeWidth="3"
          strokeLinejoin="round"
          strokeLinecap="round"
          points={line}
        />
        <circle className="hii-chart-dot" cx={last.x} cy={last.y} r="5" />
      </svg>
      <div className="hii-chart-stats">
        {chart.min != null ? <span>Min {chart.min.toFixed(1)}</span> : null}
        {chart.max != null ? <span>Max {chart.max.toFixed(1)}</span> : null}
        {chart.average != null ? <span>Avg {chart.average.toFixed(1)}</span> : null}
        {last ? <span>Latest {last.v.toFixed(1)}</span> : null}
        {chart.unit ? <span>{chart.unit}</span> : null}
      </div>
      <p className="hii-hint">{chart.notice}</p>
    </div>
  );
}
