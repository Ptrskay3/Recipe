import { useAuth } from '../utils/useAuth';

function TestProtectedRoute() {
  useAuth();

  return <div>{'this is the protected area..'}</div>;
}

export default TestProtectedRoute;
