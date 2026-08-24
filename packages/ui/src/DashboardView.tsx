import {
  previewNotice,
  sampleTimeline,
  sampleWidgets,
  timelineFilters,
} from "@healthii/dashboard";
import { Disclaimer } from "./Disclaimer";

export function DashboardView() {
  return (
    <>
      <div className="hii-page-header">
        <div>
          <p className="hii-chip">Personal health OS</p>
          <h1>What is going on with your health?</h1>
          <p className="hii-lede">
            One private place for measurements, laboratory results, documents and daily notes.
            Desktop is the workspace; mobile is for capturing life as it happens.
          </p>
        </div>
      </div>
      <div className="hii-banner" role="status">
        {previewNotice}
      </div>
      <Disclaimer />
      <section className="hii-grid" aria-label="Health overview">
        {sampleWidgets.map((widget) => (
          <article
            key={widget.id}
            className={`hii-card ${widget.id === "trend" || widget.id === "labs" ? "hii-span-2" : ""}`}
          >
            <h2>{widget.title}</h2>
            <p className="hii-metric">{widget.value}</p>
            <p className="hii-hint">{widget.hint}</p>
          </article>
        ))}
        <article className="hii-card hii-span-4">
          <h2>Health timeline</h2>
          <div className="hii-filters" role="toolbar" aria-label="Timeline filters">
            {timelineFilters.map((filter, index) => (
              <button key={filter.id} type="button" aria-pressed={index === 0}>
                {filter.label}
              </button>
            ))}
          </div>
          <div className="hii-timeline">
            {sampleTimeline.map((item) => (
              <article key={item.id} className="hii-timeline-item">
                <time>{item.dateLabel}</time>
                <div>
                  <strong>{item.title}</strong>
                  <p className="hii-hint">{item.detail}</p>
                </div>
              </article>
            ))}
          </div>
        </article>
      </section>
    </>
  );
}
