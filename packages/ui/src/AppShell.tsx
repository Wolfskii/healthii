import type { ReactNode } from "react";
import { NavLink } from "react-router-dom";
import {
  desktopNav,
  desktopSecondaryNav,
} from "@healthii/dashboard";
import { Button } from "./Button";
import { Logo } from "./Logo";
import { QuickAdd, useQuickAdd } from "./QuickAdd";
import { useTheme } from "./ThemeProvider";

export function AppShell({
  children,
  title,
}: {
  children: ReactNode;
  title?: string;
}) {
  const { theme, toggle } = useTheme();
  const quickAdd = useQuickAdd();

  return (
    <div className="hii-shell">
      <a className="hii-skip" href="#main">
        Skip to content
      </a>
      <aside className="hii-sidebar" aria-label="Primary">
        <Logo />
        <nav className="hii-nav" aria-label="Health sections">
          {desktopNav.map((item) => (
            <NavLink key={item.to} to={item.to} end={item.to === "/"}>
              {item.label}
            </NavLink>
          ))}
          <div className="hii-nav-group" aria-label="Insights">
            {desktopSecondaryNav.map((item) => (
              <NavLink key={item.to} to={item.to}>
                {item.label}
              </NavLink>
            ))}
          </div>
        </nav>
        <div className="hii-sidebar-foot">
          <nav className="hii-nav" aria-label="Account">
            <NavLink to="/settings">Settings</NavLink>
          </nav>
        </div>
      </aside>
      <div className="hii-main">
        <header className="hii-topbar">
          <input
            className="hii-search"
            type="search"
            placeholder="Search records, labs, notes…"
            aria-label="Search Healthii"
            disabled
          />
          <div className="hii-top-actions">
            <Button variant="secondary" onClick={toggle} aria-pressed={theme === "dark"}>
              {theme === "dark" ? "Light" : "Dark"}
            </Button>
            <Button onClick={quickAdd.openQuickAdd} aria-haspopup="dialog">
              Quick add
            </Button>
          </div>
        </header>
        <main id="main" className="hii-content" tabIndex={-1}>
          {title ? <h1 className="visually-hidden">{title}</h1> : null}
          {children}
        </main>
      </div>
      <QuickAdd open={quickAdd.open} onClose={quickAdd.closeQuickAdd} />
    </div>
  );
}
