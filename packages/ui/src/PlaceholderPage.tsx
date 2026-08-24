import { Disclaimer } from "./Disclaimer";

export function PlaceholderPage({
  title,
  body,
}: {
  title: string;
  body: string;
}) {
  return (
    <>
      <div className="hii-page-header">
        <div>
          <h1>{title}</h1>
          <p className="hii-lede">{body}</p>
        </div>
      </div>
      <Disclaimer />
      <section className="hii-card hii-placeholder">
        <p>This area is part of a later Healthii milestone. The navigation is in place so the product already feels like one home.</p>
      </section>
    </>
  );
}
