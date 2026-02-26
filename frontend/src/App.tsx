import { useQuery } from '@tanstack/react-query'
import './App.css'

const readGreeting = async () => {
  if (window.linxustApi?.helloFromRust) {
    return window.linxustApi.helloFromRust('LinXust')
  }
  return 'Hello, LinXust, from fallback frontend!'
}

function App() {
  const { data, isLoading } = useQuery({
    queryKey: ['hello'],
    queryFn: readGreeting,
  })

  return (
    <main className="app">
      <h1>LinXust</h1>
      <p>Electron + Rust napi-rs initialization complete.</p>
      <div className="card">
        <strong>Bridge status:</strong> {isLoading ? 'Loading...' : data}
      </div>
    </main>
  )
}

export default App
