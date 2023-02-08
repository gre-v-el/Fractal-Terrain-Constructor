# Microdocumentation

## Viewport
Action 				| Outcome 
:- 					| :-
click and drag 		| Orbit controls
scroll 				| Zoom

## Bottom Panel
Element						| Purpose
:- 							| :-
display mode radio buttons	| change the style of display
Material Threshold slider	| set the transition slope between materials
Material Smoothness slider	| set the blur between materials

## Left Panel
Element 				| Purpose
:-	 					| :-
seed drag value			| set the seed for the generation (-1 means random seed each time)
retrieve button			| set the current seed to the seed that was used to generate the currently displayed model (useful when your seed is -1 and you like the result)
reset button			| sets the seed to -1
operations list			| displays and allows for editing operations' parameters
Add Operation combo box	| shows a list of all the operations, upon selection it inserts the operation
build button			| executes the operations and displays the resulting mesh in the viewport
export obj				| saves the currently displayed terrain into an file

## Operations' UI
Every operation has its specific parameters and some buttons that are common to every operation:

Button		| Operation
:-	 		| :-
Up/Down		| Moves the operation before/after a neighboring operation
Delete		| Deletes the operation
Generate	| Generates the mesh using the operations up to this one

## Operations
* ### Add Triangle
  * ignores the previous operations (if any) and inserts a triangle
  * `size`: controls the distance between the center of the triangle and its vertex
* ### Add Triangle Square
  * ignores the previous operations (if any) and inserts a square made of two equilateral triangles
  * `size`: controls the distance between the center of the square and its vertex
* ### Add Triangle Grid
  * ignores the previous operations (if any) and inserts a grid of triangles
  * `size`: determines the side length of the entire mesh
  * `subdivisions`: determines how many triangles there will be along the edge of the mesh
* ### Add Square Grid
  * ignores the previous operations (if any) and inserts a grud of squares, each made of two equilateral triangles
  * `size`: determines the side length of the entire mesh
  * `subdivisions`: determines how many squares there will be along the edge of the mesh
* ### Subdivide
  * increases the resolution of the mesh without  changing its chape by inserting new vertices along edges
  * `iterations`: for i iterations, the number of new vertices along each edge will be $2^i - 1$
* ### Displace Random
  * changes the position of each vertex randomly
  * `amount`: the maximum distance a vertex is going to be displaced
  * `axes`: on which axes the displacement should occur (Y is up)
* ### Displace Smooth
  * changes the position of each vertex using a gradient noise
  * `amount`: the maximum distance a vertex is going to be displaced
  * `scale`: the size of a feature (larger values result in sparser hills)
  * `octaves`: how many octaves of noise are going to be added
  * `axes`: on which axes the displacement should occur (Y is up)
* ### Smooth
  * Smooths the mesh by moving each vertex towards the average position of all its neighbors
  * `amount`: how much a vertex should move (0 means no movement, 1 means all the way to the average position)
  * `iterations`: how many times the algorithm should be repeted
* ### Fractal Terrain
  * `iterations`: how much detail to add (the same as iterations for Subdivide)
  * `displacement start`: how much a vertex should be displacement on the first iteration
  * `displacement decay`: how many times the displacement of nth iteration should be lower than (n-1)th's.