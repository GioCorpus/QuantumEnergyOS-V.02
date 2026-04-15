// ═══════════════════════════════════════════════════════════════════════
//  QuantumEnergyOS Mobile — App principal
//  Dark mode nativo, gestos táctiles, 4 pantallas principales
// ═══════════════════════════════════════════════════════════════════════

import { useState, useEffect } from "react";
import { Routes, Route, useNavigate, useLocation } from "react-router-dom";
import { AnimatePresence, motion } from "framer-motion";
import { Zap, Grid, Atom, Sun, Settings } from "lucide-react";

// Screens
import DashboardScreen   from "./screens/Dashboard";
import GridScreen        from "./screens/GridBalance";
import Quartz4DScreen    from "./screens/Quartz4D";
import SolarScreen       from "./screens/Solar";
import SettingsScreen    from "./screens/SettingsScreen";

// Store global
import { useAppStore }   from "./store/appStore";

// Tauri plugins
import { isPermissionGranted, requestPermission, sendNotification }
    from "@tauri-apps/plugin-notification";

const NAV_ITEMS = [
  { path: "/",        icon: Zap,      label: "Dashboard" },
  { path: "/grid",    icon: Grid,     label: "Red"       },
  { path: "/quartz",  icon: Atom,     label: "Cuarzo 4D" },
  { path: "/solar",   icon: Sun,      label: "Solar"     },
  { path: "/settings",icon: Settings, label: "Config"   },
];

export default function App() {
  const navigate    = useNavigate();
  const location    = useLocation();
  const { init, solarRisk } = useAppStore();

  // Inicializar al montar
  useEffect(() => {
    init();
    setupNotifications();
  }, []);

  // Notificación cuando hay riesgo solar alto
  useEffect(() => {
    if (solarRisk === "HIGH" || solarRisk === "EXTREME") {
      notifyUser("⚠️ Alerta Solar",
        `Tormenta solar ${solarRisk.toLowerCase()} detectada. Red en riesgo.`);
    }
  }, [solarRisk]);

  return (
    <div className="app-shell">
      {/* Área de contenido — scroll vertical */}
      <main className="screen-area">
        <AnimatePresence mode="wait">
          <Routes location={location} key={location.pathname}>
            <Route path="/"        element={<PageWrapper><DashboardScreen /></PageWrapper>} />
            <Route path="/grid"    element={<PageWrapper><GridScreen /></PageWrapper>} />
            <Route path="/quartz"  element={<PageWrapper><Quartz4DScreen /></PageWrapper>} />
            <Route path="/solar"   element={<PageWrapper><SolarScreen /></PageWrapper>} />
            <Route path="/settings"element={<PageWrapper><SettingsScreen /></PageWrapper>} />
          </Routes>
        </AnimatePresence>
      </main>

      {/* Bottom navigation — iOS/Android style */}
      <nav className="bottom-nav">
        {NAV_ITEMS.map(({ path, icon: Icon, label }) => {
          const active = location.pathname === path;
          return (
            <button
              key={path}
              className={`nav-item ${active ? "active" : ""}`}
              onClick={() => navigate(path)}
            >
              <Icon size={22} strokeWidth={active ? 2.5 : 1.8} />
              <span className="nav-label">{label}</span>
            </button>
          );
        })}
      </nav>
    </div>
  );
}

function PageWrapper({ children }: { children: React.ReactNode }) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -8 }}
      transition={{ duration: 0.22, ease: "easeOut" }}
      className="page"
    >
      {children}
    </motion.div>
  );
}

async function setupNotifications() {
  const granted = await isPermissionGranted();
  if (!granted) {
    const perm = await requestPermission();
    if (perm !== "granted") return;
  }
}

async function notifyUser(title: string, body: string) {
  try {
    const granted = await isPermissionGranted();
    if (granted) {
      await sendNotification({ title, body, icon: "icons/128x128.png" });
    }
  } catch (e) {
    console.warn("Notification error:", e);
  }
}
