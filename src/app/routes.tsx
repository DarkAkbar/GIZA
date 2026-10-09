import { createHashRouter, Navigate } from "react-router-dom";
import { AppLayout } from "./layout/AppLayout";
import { DashboardPage } from "../pages/Dashboard/DashboardPage";
import { AnalyzePage } from "../pages/Analyze/AnalyzePage";
import { DuplicatesPage } from "../pages/Duplicates/DuplicatesPage";
import { LargeFilesPage } from "../pages/LargeFiles/LargeFilesPage";
import { CleanupPage } from "../pages/Cleanup/CleanupPage";
import { OrganizePage } from "../pages/Organize/OrganizePage";
import { HistoryPage } from "../pages/History/HistoryPage";
import { SettingsPage } from "../pages/Settings/SettingsPage";

export const router = createHashRouter([
  {
    path: "/",
    element: <AppLayout />,
    children: [
      { index: true, element: <Navigate to="/dashboard" replace /> },
      { path: "dashboard", element: <DashboardPage /> },
      { path: "analyze", element: <AnalyzePage /> },
      { path: "duplicates", element: <DuplicatesPage /> },
      { path: "large-files", element: <LargeFilesPage /> },
      { path: "cleanup", element: <CleanupPage /> },
      { path: "organize", element: <OrganizePage /> },
      { path: "history", element: <HistoryPage /> },
      { path: "settings", element: <SettingsPage /> },
    ],
  },
]);
