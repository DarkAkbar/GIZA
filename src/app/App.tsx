import { RouterProvider } from "react-router-dom";
import { SplashGate } from "../components/SplashGate";
import { router } from "./routes";

export function App() {
  return (
    <SplashGate>
      <RouterProvider router={router} />
    </SplashGate>
  );
}
