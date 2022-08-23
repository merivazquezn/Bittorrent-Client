
export class API {
    get(path, queryParams) {
        let url = "";
        if (queryParams) {
            url = 'http://localhost:7878/' + path + '?' + this.queryString(queryParams);
        } else {
            url = 'http://localhost:7878/' + path;
        }
        return fetch(url, {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
                Accept: 'application/json'
            }
        })
            .then((response) => {
                if (response.ok) {
                    return response.json();
                } else {
                    return response.json().then((json) => {
                        throw json.error;
                    });
                }
            })
            .catch((error) => {
                console.error(error);
                throw new Error(`${error}`);
            });
    }

    post(path, body) {
        const url = 'http://localhost:7878/' + path;
        return fetch(url, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                Accept: 'application/json'
            },
            body: JSON.stringify(body)
        })
            .then((response) => {
                if (response.ok) {
                    return response.json();
                } else {
                    return response.json().then((json) => {
                        throw json.error;
                    });
                }
            })
            .catch((error) => {
                console.error(error);
                throw new Error(`${error}`);
            });
    }

    queryString(queryParams) {
        if (!queryParams) {
            return '';
        }

        return Object.keys(queryParams)
            .map((key) => key + '=' + queryParams[key])
            .join('&');
    }
}
