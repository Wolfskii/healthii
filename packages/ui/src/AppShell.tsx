import { useEffect, type FormEvent, type ReactNode } from "react";
import { NavLink, useNavigate } from "react-router-dom";
import {
  desktopNav,
  desktopSecondaryNav,
} from "@healthii/dashboard";
import { Button } from "./Button";
import { Logo } from "./Logo";
import { QuickAdd, useQuickAdd } from "./QuickAdd";
import { useOptionalSession } from "./session";
import { useTheme } from "./ThemeProvider";

export function AppShell({
  children,
  title,
}: {
  children: ReactNode;
  title?: string;
}) {
  const { theme, toggle } = useTheme();
  const { open, openQuickAdd, closeQuickAdd } = useQuickAdd();
  const session = useOptionalSession();
  const navigate = useNavigate();

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        openQuickAdd();
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [openQuickAdd]);

  function onSearch(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    const q = String(data.get("q") ?? "").trim();
    if (q.length >= 2) {
      navigate(`/search?q=${encodeURIComponent(q)}`);
    }
  }

  async function signOut() {
    if (!session) {
      return;
    }
    await session.api.logout().catch(() => undefined);
    session.setSession(null, null);
  }

  return (
    <div className="hii-shell">
      <a className="hii-skip" href="#main">
        Skip to content
      </a>
      <aside className="hii-sidebar" aria-label="Primary">
        <Logo />
        {session?.user ? <p className="hii-sidebar-user">{session.user.display_name}</p> : null}
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
          <form className="hii-search-form" onSubmit={onSearch} role="search">
            <input
              className="hii-search"
              type="search"
              name="q"
              placeholder="Search records, labs, notes…"
              aria-label="Search Healthii"
              minLength={2}
              disabled={!session?.token}
            />
          </form>
          <div className="hii-top-actions">
            <Button
              variant="secondary"
              onClick={toggle}
              aria-pressed={theme === "dark"}
              title={theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
            >
              {theme === "dark" ? "Light" : "Dark"}
            </Button>
            {session?.token ? (
              <Button variant="secondary" type="button" onClick={signOut}>
                Sign out
              </Button>
            ) : null}
            <Button
              onClick={openQuickAdd}
              aria-haspopup="dialog"
              title="Quick add (Ctrl or Cmd + K)"
            >
              Quick add
              <kbd className="hii-kbd">Ctrl+K</kbd>
            </Button>
          </div>
        </header>
        <main id="main" className="hii-content" tabIndex={-1}>
          {title ? <h1 className="visually-hidden">{title}</h1> : null}
          {children}
        </main>
      </div>
      <nav className="hii-mobile-nav" aria-label="Health sections">
        {desktopNav.map((item) => (
          <NavLink key={item.to} to={item.to} end={item.to === "/"}>
            {item.label}
          </NavLink>
        ))}
        <NavLink to="/insights">Insights</NavLink>
        <NavLink to="/settings">Settings</NavLink>
      </nav>
      <QuickAdd open={open} onClose={closeQuickAdd} />
    </div>
  );
}
