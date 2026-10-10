module Example where
data Shade = Light | Dark
type Name = String
class Render a where
  render :: a -> String
double :: Int -> Int
double x = x * 2
café x = x
