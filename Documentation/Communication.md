
## Pi side:
``` 
/get-fingerprint/id - GET Request at port 8000 
	returns 200 OK if enrollment mode was successfully activated
```


## Server side:
``` 
/enroll-finished/id - POST Request at port 8000 
	The Pi POSTS the fingerprint-file to this route
	Returns Status Code 200 OK if the request was successfull
```

``` 
/enroll-error/id - POST Request at port 8000 
	The Pi POSTS here if an error has occured wether it was 
	the enrollment mode failing or the scanning was not done
	correctly.
	Returns Status Code 200 OK if the request was successfull
```