## Basic architechture:
```No_this_is_not_generated_with_ai 
								---------
								|Main Pi|
								---------
									|
					_________________________________
					|               |               |
				 Door_01         Door_02         Door_03
```


## Workflow: 

- Detect change (f.e. a user gets added)
- Send a http request to each door pi to fetch the new folder
- Replace the old folder with the new one but keep the old one as backup